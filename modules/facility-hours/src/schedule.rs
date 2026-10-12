//! Structured opening hours, and what they say right now (§2.6).
//!
//! A row used to carry one free-text line — `"08:00 – 20:00"`, `"à partir de 16:00"`, `"24 h/24"`.
//! That reads fine and computes nothing: "Ouvert", "Ouvre à 08:00" and "Fermé" cannot be derived
//! from a sentence, and neither can Monday-to-Sunday.
//!
//! So a row may now carry times as well. **May**, not must: hosts have already written their hours
//! in prose, and nothing here throws that away. A row without structured times keeps showing its
//! sentence exactly as before, and gains no live state — which is honest, rather than a guess at
//! what "à partir de 16:00" means.
//!
//! Times are `HH:MM` in the property's timezone. A closing time earlier than its opening spans
//! midnight: `22:00 – 02:00` is open at one in the morning, and saying otherwise would put a guest
//! outside a door that is in fact open.

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::host::time::PropertyTz;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::hours::parse_range;
use serde::{Deserialize, Serialize};

use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc, Weekday};

/// Minutes since midnight, or `None` when the text is not `HH:MM`.
///
/// Deliberately strict: a lenient parser would read "à partir de 16:00" as 16:00 and drop the
/// "à partir de", turning a sentence the host wrote into a schedule they did not.
pub fn parse_hm(raw: &str) -> Option<u32> {
    let text = raw.trim();
    let (hours, minutes) = text.split_once(':')?;
    if hours.len() > 2 || minutes.len() != 2 {
        return None;
    }
    let hours: u32 = hours.parse().ok()?;
    let minutes: u32 = minutes.parse().ok()?;
    if hours > 23 || minutes > 59 {
        return None;
    }
    Some(hours * 60 + minutes)
}

/// Une fermeture exceptionnelle datée (§2.2) — et, sur une ligne d'avant, une dérogation d'un jour
/// de semaine (`day`) ou une ligne « selon le jour » (`id`).
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DayHours {
    /// La date, `AAAA-MM-JJ`, celle qu'écrit le `DatePicker`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub date: String,
    /// Fermé ce jour-là, quelle que soit la plage.
    pub closed: bool,
    /// La plage de ce jour, `HH:MM-HH:MM`, quand il n'est pas fermé.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub range: String,
    /// Le motif, montré au voyageur le jour même.
    pub reason: I18nText,
    /// Ancien : `mon` … `sun`, une dérogation hebdomadaire.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub day: String,
    /// Ancien : le jour d'une ligne « selon le jour », envoyé par le champ caché `….id` de
    /// l'ancien formulaire.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opens_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closes_at: Option<String>,
}

impl DayHours {
    /// Le jour de semaine d'une ancienne ligne (`id`, sinon `day`) ; `None` pour une fermeture
    /// datée.
    pub fn weekday(&self) -> Option<usize> {
        week_index(&self.id).or_else(|| week_index(&self.day))
    }

    /// La date, quand elle en est une.
    pub fn date(&self) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(self.date.trim(), "%Y-%m-%d").ok()
    }

    /// Les plages de ce jour : aucune s'il est fermé, ou sans plage lisible.
    pub fn spans(&self) -> Vec<DaySpan> {
        if self.closed {
            return Vec::new();
        }
        parse_range(&self.range)
            .and_then(|(start, end)| {
                Some(DaySpan {
                    opens: parse_hm(&start)?,
                    closes: parse_hm(&end)?,
                })
            })
            .into_iter()
            .collect()
    }

    /// Rien de saisi : une ligne qu'« Ajouter » vient d'ouvrir.
    pub fn is_blank(&self) -> bool {
        self.date.trim().is_empty() && self.range.trim().is_empty() && self.reason.is_blank()
    }
}

/// The seven days, in the order a week is read.
pub const WEEK: [Weekday; 7] = [
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
    Weekday::Sat,
    Weekday::Sun,
];

/// L'index dans [`WEEK`] d'une clé `mon` … `sun`, la casse ignorée.
pub fn week_index(raw: &str) -> Option<usize> {
    WEEK.iter()
        .position(|day| raw.trim().eq_ignore_ascii_case(day_key(*day)))
}

/// The `mon` … `sun` key for a weekday.
pub fn day_key(day: Weekday) -> &'static str {
    match day {
        Weekday::Mon => "mon",
        Weekday::Tue => "tue",
        Weekday::Wed => "wed",
        Weekday::Thu => "thu",
        Weekday::Fri => "fri",
        Weekday::Sat => "sat",
        Weekday::Sun => "sun",
    }
}

/// What a facility is doing at a given moment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// Open around the clock — no times to compare.
    AlwaysOpen,
    /// Ouvert, et ferme à cette heure (minutes après minuit) : « Ouvert · ferme à 19:00 » (§9 #2).
    Open(u32),
    /// Ouvert, mais ferme dans ce nombre de minutes ([`CLOSES_SOON`] au plus) : « Ferme dans
    /// 40 min » (spec Horaires §9 #7).
    ClosesSoon(u32),
    /// Shut for now; opens at this many minutes past midnight, today.
    OpensAt(u32),
    Closed,
    /// Hors saison : la ligne rouvrira au début de la saison, « Fermé jusqu'au 1 juin » (§9 #4).
    /// `None` quand ce début n'existe pas cette année-là (un `04-31` que l'hôte a saisi).
    OutOfSeason(Option<NaiveDate>),
}

/// À partir de combien de minutes avant la fermeture « Ouvert » devient « Ferme dans N min ».
///
/// La spec n'en donne que l'exemple, 40 min : une heure le couvre, et prévient assez tôt pour
/// ne pas arriver devant une porte qui se ferme.
pub const CLOSES_SOON: u32 = 60;

/// « Ouvert », ou « Ferme dans N min » quand il reste [`CLOSES_SOON`] minutes ou moins.
fn open_for(span: &DaySpan, minutes: u32) -> State {
    let remaining = span.remaining(minutes);
    if remaining <= CLOSES_SOON {
        State::ClosesSoon(remaining)
    } else {
        State::Open(span.closes)
    }
}

/// The opening and closing of one day, once exceptions are applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DaySpan {
    pub opens: u32,
    pub closes: u32,
}

impl DaySpan {
    /// The span runs past midnight (`22:00 – 02:00`).
    pub fn overnight(&self) -> bool {
        self.closes <= self.opens
    }

    /// La coupure `from` → `to` tient dans la plage : elle la coupe en deux plages qui ne se
    /// chevauchent pas. Jamais sur une plage qui passe minuit — un bar ouvert de 22 h à 2 h ne
    /// déjeune pas.
    pub fn holds_break(&self, from: u32, to: u32) -> bool {
        !self.overnight() && self.opens < from && from < to && to < self.closes
    }

    /// Les minutes qui restent avant la fermeture, à `minutes` d'une plage qui le couvre.
    pub fn remaining(&self, minutes: u32) -> u32 {
        if self.overnight() && minutes >= self.opens {
            24 * 60 - minutes + self.closes
        } else {
            self.closes - minutes
        }
    }

    /// `minutes` falls inside the span, midnight crossing included.
    pub fn covers(&self, minutes: u32) -> bool {
        if self.overnight() {
            minutes >= self.opens || minutes < self.closes
        } else {
            minutes >= self.opens && minutes < self.closes
        }
    }
}

/// The hours a facility keeps, as the host filled them in.
#[derive(Debug, Clone, Default)]
pub struct Schedule {
    pub all_day: bool,
    /// « Sur demande » : pas d'heures, mais la saison, les jours de fermeture et les fermetures
    /// exceptionnelles disent encore « Fermé ».
    pub on_request: bool,
    /// Des heures à calculer : sans elles, la ligne garde sa phrase et n'affiche pas d'état.
    pub structured: bool,
    /// Les plages de chaque jour, lundi d'abord — une, deux ou trois (§9 #3) ; vide, le jour est
    /// fermé. Ignorées 24 h/24.
    pub week: [Vec<DaySpan>; 7],
    /// Les jours de fermeture cochés.
    pub closed: [bool; 7],
    /// Les fermetures exceptionnelles : leur date, et ses plages (aucune : fermé). Elles passent
    /// avant l'horaire habituel (§2.2).
    pub dated: Vec<(NaiveDate, Vec<DaySpan>)>,
    /// La saison, en nombres `MMJJ` : hors de ces dates, la ligne est fermée quoi que disent ses
    /// heures. Sans année, et elle peut passer l'hiver (`11-15` → `03-15`).
    pub season: Option<(u32, u32)>,
}

impl Schedule {
    /// Whether anything here can be computed. A row without times keeps its sentence.
    pub fn is_structured(&self) -> bool {
        self.structured
    }

    /// Les plages habituelles d'un jour de semaine ; vide quand il est fermé.
    pub fn spans_on(&self, day: Weekday) -> Vec<DaySpan> {
        self.week[day.num_days_from_monday() as usize].clone()
    }

    /// Ce jour est coché fermé.
    pub fn closed_on(&self, day: Weekday) -> bool {
        self.closed[day.num_days_from_monday() as usize]
    }

    /// La fermeture exceptionnelle de `date`, s'il y en a une.
    pub fn exception_on(&self, date: NaiveDate) -> Option<&[DaySpan]> {
        self.dated
            .iter()
            .find(|(day, _)| *day == date)
            .map(|(_, spans)| spans.as_slice())
    }

    /// Les plages d'une date : sa fermeture exceptionnelle d'abord, sinon son jour de semaine.
    /// Vide 24 h/24 : il n'y a pas de plage, et rien ne déborde sur le lendemain.
    pub fn spans_at(&self, date: NaiveDate) -> Vec<DaySpan> {
        match self.exception_on(date) {
            Some(spans) => spans.to_vec(),
            None if self.all_day => Vec::new(),
            None => self.spans_on(date.weekday()),
        }
    }

    /// What the facility is doing at `now`, read in the property's timezone.
    ///
    /// Yesterday is consulted before today: at one in the morning, a `22:00 – 02:00` facility is
    /// still inside *yesterday's* span, and looking only at today would send the guest away from an
    /// open door.
    pub fn state_at(&self, now: DateTime<Utc>, tz: Option<&PropertyTz>) -> Option<State> {
        if !self.structured && !self.on_request {
            return None;
        }
        let local = local_time(now, tz);

        // La saison d'abord : un sauna ouvert de 17 h à 21 h en juillet est fermé en janvier, et
        // annoncer « Ouvert » parce que l'heure colle envoie le voyageur devant une porte close.
        if !self.in_season(&local) {
            return Some(State::OutOfSeason(self.season_start_after(local.date())));
        }
        let date = local.date();
        if self.exception_on(date).is_none() {
            if self.on_request {
                // Sur demande, rien à calculer — sauf un jour fermé.
                return self.closed_on(date.weekday()).then_some(State::Closed);
            }
            if self.all_day {
                // Ouvert 24 h/24, sauf le jour de fermeture que l'hôte a coché.
                return Some(if self.closed_on(date.weekday()) {
                    State::Closed
                } else {
                    State::AlwaysOpen
                });
            }
        }
        let minutes = local.hour() * 60 + local.minute();

        if let Some(yesterday) = date.pred_opt() {
            if let Some(span) = self
                .spans_at(yesterday)
                .iter()
                .find(|span| span.overnight() && minutes < span.closes)
            {
                return Some(open_for(span, minutes));
            }
        }

        let spans = self.spans_at(date);
        if let Some(span) = spans.iter().find(|span| span.covers(minutes)) {
            return Some(open_for(span, minutes));
        }
        // La prochaine ouverture du jour : après le déjeuner, c'est la seconde plage qu'on
        // annonce, pas celle du matin qui vient de fermer.
        if let Some(next) = spans
            .iter()
            .map(|span| span.opens)
            .filter(|&opens| minutes < opens)
            .min()
        {
            return Some(State::OpensAt(next));
        }
        Some(State::Closed)
    }

    /// Le prochain début de saison après `today` : cette année, ou l'an prochain s'il est passé.
    fn season_start_after(&self, today: NaiveDate) -> Option<NaiveDate> {
        let (from, _) = self.season?;
        let start = |year| NaiveDate::from_ymd_opt(year, from / 100, from % 100);
        start(today.year())
            .filter(|start| *start > today)
            .or_else(|| start(today.year() + 1))
    }

    /// Ce jour tombe-t-il dans la saison — toujours vrai quand l'hôte n'en a pas donné.
    fn in_season(&self, local: &chrono::NaiveDateTime) -> bool {
        let Some((from, to)) = self.season else {
            return true;
        };
        let day = month_day(local);
        if from <= to {
            (from..=to).contains(&day)
        } else {
            // La saison passe l'hiver : elle court de `from` à la fin de l'année, puis du début
            // de l'année à `to`.
            day >= from || day <= to
        }
    }
}

/// `now` à l'heure du logement — à l'heure UTC sans fuseau connu.
pub fn local_time(now: DateTime<Utc>, tz: Option<&PropertyTz>) -> chrono::NaiveDateTime {
    match tz {
        Some(tz) => tz.to_local(now).naive_local(),
        None => now.naive_utc(),
    }
}

/// `MM-JJ` en un nombre comparable : `0415` pour le 15 avril.
fn month_day(local: &chrono::NaiveDateTime) -> u32 {
    use chrono::Datelike;
    local.month() * 100 + local.day()
}

/// `JJ/MM` (le formulaire) ou `MM-JJ` (une ligne d'avant) → le même nombre comparable, ou `None`
/// quand ce n'est pas une date du calendrier.
pub fn parse_month_day(raw: &str) -> Option<u32> {
    let raw = raw.trim();
    let (month, day) = match raw.split_once('/') {
        Some((day, month)) => (month, day),
        None => raw.split_once('-')?,
    };
    let month: u32 = month.trim().parse().ok()?;
    let day: u32 = day.trim().parse().ok()?;
    // 31 partout : février à 31 jours ne ferme rien de travers, et refuser le 31 janvier parce
    // qu'un mois voisin est plus court serait pire.
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(month * 100 + day)
}

/// Ce qu'un état dit, et sur quel ton — le badge du livret et la colonne « Aujourd'hui » du
/// tableau hôte. `None` 24 h/24 : « 24 h/24 » se lit ailleurs, sans statut (§9 #1).
pub fn state_label(state: &State, locale: &str) -> Option<(String, Tone)> {
    Some(match state {
        State::AlwaysOpen => return None,
        // « Ouvert · ferme à 02:00 » (§9 #2) : ouvert, et jusqu'à quand.
        State::Open(closes) => (
            t!("guest.state.openUntil", time = format_minutes(*closes))
                .unwrap_or_else(|_| "i18n:guest.state.open".to_string()),
            Tone::Success,
        ),
        // « Ferme dans 40 min » (§9 #7) : ouvert, mais plus pour longtemps.
        State::ClosesSoon(minutes) => (
            t!("guest.state.closesSoon", minutes = minutes)
                .unwrap_or_else(|_| "i18n:guest.state.open".to_string()),
            Tone::Warning,
        ),
        State::OpensAt(minutes) => (
            t!("guest.state.opensAt", time = format_minutes(*minutes))
                .unwrap_or_else(|_| "i18n:guest.state.opensAtPlain".to_string()),
            Tone::Neutral,
        ),
        State::Closed => ("i18n:guest.state.closed".to_string(), Tone::Neutral),
        // « Fermé jusqu'au 1er juin » (§9 #4) : quand revenir, et pas seulement que c'est fermé.
        State::OutOfSeason(reopens) => (
            reopens
                .and_then(|date| {
                    t!("guest.state.closedUntil", date = reopen_date(date, locale)).ok()
                })
                .unwrap_or_else(|| "i18n:guest.state.outOfSeason".to_string()),
            Tone::Neutral,
        ),
    })
}

/// « 1er juin » en français, la date courte ailleurs.
fn reopen_date(date: NaiveDate, locale: &str) -> String {
    let short = portaki_sdk::host::time::short_date(date, locale);
    match (locale.starts_with("fr"), short.strip_prefix("1 ")) {
        (true, Some(rest)) if date.day() == 1 => format!("1er {rest}"),
        _ => short,
    }
}

/// « 08:30 – 12:00 · 14:00 – 18:00 » : les plages d'un jour, vide quand il est fermé.
pub fn spans_text(spans: &[DaySpan]) -> String {
    spans
        .iter()
        .map(|span| {
            format!(
                "{} – {}",
                format_minutes(span.opens),
                format_minutes(span.closes)
            )
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

/// La saison au format du formulaire, `JJ/MM` ; une valeur illisible reste telle quelle.
pub fn season_display(raw: &str) -> String {
    match parse_month_day(raw) {
        Some(value) => format!("{:02}/{:02}", value % 100, value / 100),
        None => raw.trim().to_string(),
    }
}

/// `HH:MM` depuis des minutes après minuit.
pub fn format_minutes(minutes: u32) -> String {
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(hour: u32, minute: u32) -> DateTime<Utc> {
        // A Wednesday.
        Utc.with_ymd_and_hms(2026, 9, 30, hour, minute, 0).unwrap()
    }

    /// Une saison d'avril à octobre ferme en janvier ; une saison de novembre à mars l'ouvre.
    #[test]
    fn a_season_closes_outside_its_months_and_may_cross_the_winter() {
        let summer = Schedule {
            season: (parse_month_day("04-01")).zip(parse_month_day("10-31")),
            ..day("09:00", "20:00")
        };
        let winter = Schedule {
            season: (parse_month_day("11-15")).zip(parse_month_day("03-15")),
            ..day("09:00", "20:00")
        };
        let january = Utc.with_ymd_and_hms(2026, 1, 20, 10, 0, 0).unwrap();
        let july = Utc.with_ymd_and_hms(2026, 7, 20, 10, 0, 0).unwrap();

        let date = |y, m, d| NaiveDate::from_ymd_opt(y, m, d);
        assert_eq!(
            summer.state_at(january, None),
            Some(State::OutOfSeason(date(2026, 4, 1)))
        );
        assert_eq!(summer.state_at(july, None), Some(State::Open(20 * 60)));
        assert_eq!(winter.state_at(january, None), Some(State::Open(20 * 60)));
        assert_eq!(
            winter.state_at(july, None),
            Some(State::OutOfSeason(date(2026, 11, 15)))
        );
        // Passé le début de cette année, c'est celui de l'an prochain qui rouvre.
        let december = Utc.with_ymd_and_hms(2026, 12, 20, 10, 0, 0).unwrap();
        assert_eq!(
            summer.state_at(december, None),
            Some(State::OutOfSeason(date(2027, 4, 1)))
        );
        // Un début qui n'existe pas (le 31 avril) ne donne pas de date, seulement « Hors saison ».
        let impossible = Schedule {
            season: parse_month_day("04-31").zip(parse_month_day("10-31")),
            ..day("09:00", "20:00")
        };
        assert_eq!(
            impossible.state_at(january, None),
            Some(State::OutOfSeason(None))
        );
    }

    /// Une heure avant la fermeture, « Ouvert » devient « Ferme dans N min » (§9 #7) : en fin de
    /// journée, avant la coupure, et sur une plage qui passe minuit, des deux côtés de minuit.
    #[test]
    fn closing_within_the_hour_says_how_long_is_left() {
        let pool = day("08:00", "20:00");
        assert_eq!(pool.state_at(at(18, 59), None), Some(State::Open(20 * 60)));
        assert_eq!(pool.state_at(at(19, 0), None), Some(State::ClosesSoon(60)));
        assert_eq!(pool.state_at(at(19, 20), None), Some(State::ClosesSoon(40)));

        let reception = day_with_break("08:30", "18:00", "12:00", "14:00");
        assert_eq!(
            reception.state_at(at(11, 30), None),
            Some(State::ClosesSoon(30))
        );

        let bar = day("22:00", "02:00");
        assert_eq!(bar.state_at(at(23, 30), None), Some(State::Open(2 * 60)));
        assert_eq!(bar.state_at(at(1, 30), None), Some(State::ClosesSoon(30)));
        let late = day("20:00", "00:30");
        assert_eq!(late.state_at(at(23, 45), None), Some(State::ClosesSoon(45)));
    }

    /// Une seule date ne fait pas une saison : la ligne reste de toute saison.
    #[test]
    fn a_month_day_that_is_not_one_is_refused() {
        assert_eq!(parse_month_day("04-01"), Some(401));
        assert_eq!(parse_month_day("13-01"), None);
        assert_eq!(parse_month_day("04-32"), None);
        assert_eq!(parse_month_day("avril"), None);
        assert_eq!(parse_month_day(""), None);
    }

    /// Les mêmes plages tous les jours.
    fn every_day(spans: &[(&str, &str)]) -> Schedule {
        let spans: Vec<DaySpan> = spans
            .iter()
            .map(|(opens, closes)| DaySpan {
                opens: parse_hm(opens).unwrap(),
                closes: parse_hm(closes).unwrap(),
            })
            .collect();
        Schedule {
            structured: true,
            week: std::array::from_fn(|_| spans.clone()),
            ..Schedule::default()
        }
    }

    fn day(opens: &str, closes: &str) -> Schedule {
        every_day(&[(opens, closes)])
    }

    /// La même journée, avec une coupure : deux plages.
    fn day_with_break(opens: &str, closes: &str, from: &str, to: &str) -> Schedule {
        every_day(&[(opens, from), (to, closes)])
    }

    #[test]
    fn a_time_is_read_only_when_it_is_one() {
        assert_eq!(parse_hm("08:00"), Some(480));
        assert_eq!(parse_hm(" 8:05 "), Some(485));
        assert_eq!(parse_hm("23:59"), Some(1439));
        // The prose a host may have written stays prose.
        assert_eq!(parse_hm("à partir de 16:00"), None);
        assert_eq!(parse_hm("24 h/24"), None);
        assert_eq!(parse_hm("8h"), None);
        assert_eq!(parse_hm("24:00"), None);
        assert_eq!(parse_hm("08:60"), None);
    }

    #[test]
    fn a_row_without_times_computes_nothing() {
        let prose = Schedule::default();
        assert!(!prose.is_structured());
        assert_eq!(prose.state_at(at(10, 0), None), None);
    }

    #[test]
    fn open_before_closing_and_shut_after() {
        let pool = day("08:00", "20:00");
        assert_eq!(pool.state_at(at(7, 0), None), Some(State::OpensAt(480)));
        assert_eq!(pool.state_at(at(8, 0), None), Some(State::Open(20 * 60)));
        assert_eq!(pool.state_at(at(19, 59), None), Some(State::ClosesSoon(1)));
        assert_eq!(pool.state_at(at(20, 0), None), Some(State::Closed));
    }

    #[test]
    fn a_bar_that_closes_after_midnight_is_open_at_one() {
        let bar = day("22:00", "02:00");
        assert_eq!(bar.state_at(at(23, 0), None), Some(State::Open(2 * 60)));
        // One in the morning belongs to yesterday's span, not to a day that has not opened.
        assert_eq!(bar.state_at(at(1, 0), None), Some(State::ClosesSoon(60)));
        // À trois heures il est fermé, et il rouvre ce soir : « Ouvre à 22:00 » en dit plus que
        // « Fermé », et c'est ce que le §2.6 veut afficher.
        assert_eq!(bar.state_at(at(3, 0), None), Some(State::OpensAt(1320)));
    }

    #[test]
    fn a_closed_day_is_closed_whatever_the_usual_hours() {
        let mut gym = day("06:00", "22:00");
        gym.week[2].clear();
        gym.closed[2] = true;
        assert_eq!(gym.state_at(at(10, 0), None), Some(State::Closed));
        assert!(gym.spans_on(Weekday::Wed).is_empty());
        assert!(!gym.spans_on(Weekday::Thu).is_empty());
    }

    #[test]
    fn a_day_may_keep_its_own_hours() {
        let mut spa = day("10:00", "19:00");
        spa.week[2] = vec![DaySpan {
            opens: 14 * 60,
            closes: 18 * 60,
        }];
        assert_eq!(spa.state_at(at(11, 0), None), Some(State::OpensAt(840)));
        assert_eq!(spa.state_at(at(15, 0), None), Some(State::Open(18 * 60)));
    }

    #[test]
    fn around_the_clock_needs_no_times() {
        let parking = Schedule {
            all_day: true,
            structured: true,
            ..Schedule::default()
        };
        assert!(parking.is_structured());
        assert_eq!(parking.state_at(at(3, 0), None), Some(State::AlwaysOpen));
    }

    /// Une réception qui ferme pour déjeuner : deux plages, et « Ouvert » ne ment plus à 13 h.
    #[test]
    fn a_lunch_break_closes_the_door_and_says_when_it_reopens() {
        let schedule = day_with_break("08:30", "18:00", "12:00", "14:00");
        let spans = schedule.spans_on(Weekday::Mon);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].closes, 12 * 60);
        assert_eq!(spans[1].opens, 14 * 60);

        let at = |h: u32, m: u32| {
            schedule.state_at(
                chrono::Utc
                    .with_ymd_and_hms(2026, 6, 15, h, m, 0)
                    .single()
                    .expect("date"),
                None,
            )
        };
        assert_eq!(at(10, 0), Some(State::Open(12 * 60)));
        assert_eq!(at(13, 0), Some(State::OpensAt(14 * 60)));
        assert_eq!(at(15, 0), Some(State::Open(18 * 60)));
        assert_eq!(at(19, 0), Some(State::Closed));
        assert_eq!(at(7, 0), Some(State::OpensAt(8 * 60 + 30)));
    }

    /// Trois plages le même jour (§9 #3) : ouvert dans chacune, la suivante annoncée entre deux.
    #[test]
    fn three_ranges_a_day() {
        let shop = every_day(&[("08:00", "10:00"), ("12:00", "14:00"), ("18:00", "22:00")]);
        assert_eq!(shop.state_at(at(9, 0), None), Some(State::ClosesSoon(60)));
        assert_eq!(
            shop.state_at(at(11, 0), None),
            Some(State::OpensAt(12 * 60))
        );
        assert_eq!(
            shop.state_at(at(15, 0), None),
            Some(State::OpensAt(18 * 60))
        );
        assert_eq!(shop.state_at(at(19, 0), None), Some(State::Open(22 * 60)));
        assert_eq!(shop.state_at(at(23, 0), None), Some(State::Closed));
    }

    /// Une fermeture exceptionnelle passe avant l'horaire habituel (§2.2), 24 h/24 compris ; une
    /// plage exceptionnelle remplace celles du jour.
    #[test]
    fn a_dated_exception_wins_over_the_usual_hours() {
        let wednesday = NaiveDate::from_ymd_opt(2026, 9, 30).unwrap();
        let mut pool = day("08:00", "20:00");
        pool.dated = vec![(wednesday, Vec::new())];
        assert_eq!(pool.state_at(at(10, 0), None), Some(State::Closed));
        let late = vec![DaySpan {
            opens: 14 * 60,
            closes: 16 * 60,
        }];
        pool.dated = vec![(wednesday, late.clone())];
        assert_eq!(
            pool.state_at(at(10, 0), None),
            Some(State::OpensAt(14 * 60))
        );
        let parking = Schedule {
            all_day: true,
            structured: true,
            dated: vec![(wednesday, Vec::new())],
            ..Schedule::default()
        };
        assert_eq!(parking.state_at(at(10, 0), None), Some(State::Closed));
        let desk = Schedule {
            on_request: true,
            dated: vec![(wednesday, Vec::new())],
            ..Schedule::default()
        };
        assert_eq!(desk.state_at(at(10, 0), None), Some(State::Closed));
        assert_eq!(
            desk.state_at(at(10, 0) + chrono::Duration::days(1), None),
            None
        );
    }

    /// Le formulaire écrit `JJ/MM`, une ligne d'avant `MM-JJ` : les deux sont le 1er juin.
    #[test]
    fn a_season_reads_both_formats() {
        assert_eq!(parse_month_day("01/06"), Some(601));
        assert_eq!(parse_month_day("06-01"), Some(601));
        assert_eq!(parse_month_day("31/12"), Some(1231));
        assert_eq!(parse_month_day("32/01"), None);
        assert_eq!(season_display("06-01"), "01/06");
        assert_eq!(season_display("01/06"), "01/06");
    }
}
