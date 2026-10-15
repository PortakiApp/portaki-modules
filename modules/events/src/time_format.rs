//! Event datetime parsing and guest display labels.

use chrono::{DateTime, Duration, NaiveDateTime, Timelike, Utc};
use portaki_sdk::prelude::*;

use portaki_sdk::context::StayContext;
use portaki_sdk::host::time::{relative_day, short_date, PropertyTz, RelativeDay};

use crate::config::EventRow;

pub fn parse_starts_at(raw: &str) -> Option<DateTime<Utc>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = trimmed.parse::<DateTime<Utc>>() {
        return Some(dt);
    }
    parse_wall(trimmed).map(|naive| naive.and_utc())
}

/// Une date sans fuseau (`2026-08-15T22:30:00`) : l'heure du mur, celle des sélecteurs du
/// formulaire comme de l'ancien champ texte.
fn parse_wall(raw: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S"))
        .ok()
}

/// L'événement avec ses heures du mur portées en instants, à l'heure du logement : 22 h 30 à
/// Paris l'été, c'est 20 h 30 UTC. Une seule fois, au chargement, pour que la fenêtre, « Ce soir »
/// et la répétition comparent des instants, et que l'affichage les ramène à l'heure du logement
/// sans décalage. Une date qui porte son fuseau (`…Z`) est déjà un instant ; sans fuseau connu,
/// rien ne bouge (UTC, comme avant).
pub fn anchored(mut event: EventRow, tz: Option<&PropertyTz>) -> EventRow {
    let Some(tz) = tz else {
        return event;
    };
    let anchor = |raw: &str| {
        let raw = raw.trim();
        DateTime::parse_from_rfc3339(raw)
            .is_err()
            .then(|| parse_wall(raw))
            .flatten()
            .map(|wall| tz.from_local(wall).to_rfc3339())
    };
    if let Some(starts) = anchor(&event.starts_at) {
        event.starts_at = starts;
    }
    if let Some(ends) = event.ends_at.as_deref().and_then(anchor) {
        event.ends_at = Some(ends);
    }
    event
}

/// « 22:30 » : l'heure de l'instant, à l'heure du logement (UTC sans fuseau connu).
pub fn local_time(at: DateTime<Utc>, tz: Option<&PropertyTz>) -> String {
    crate::config::local(at, tz).format("%H:%M").to_string()
}

/// La fenêtre du séjour, élargie d'un jour de chaque côté (§0.7).
///
/// « Rien avant, rien après » : un séjour de trois nuits n'a pas à montrer le festival du mois
/// prochain, et la liste se remplissait pourtant de tout ce que l'agenda avait à dire. La veille
/// de l'arrivée et le lendemain du départ restent dedans — on prépare la veille, et on part après
/// le petit-déjeuner.
///
/// Sans dates de séjour — un aperçu, une surface hors séjour — rien n'est écarté : mieux vaut la
/// liste entière qu'une fenêtre inventée autour d'une date qu'on n'a pas.
pub fn stay_window(stay: Option<&StayContext>) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let stay = stay?;
    let from = stay.checkin_at? - Duration::days(1);
    let to = stay.checkout_at? + Duration::days(1);
    (from <= to).then_some((from, to))
}

/// Les événements qui tombent dans cette fenêtre, dans l'ordre reçu.
///
/// Un événement sans date lisible est gardé : il vient de l'hôte, qui a écrit quelque chose, et
/// l'écarter sur une date qu'on n'a pas su lire effacerait son travail.
///
/// Un événement qui se répète y tombe à sa prochaine occurrence depuis `now`, ou depuis l'ouverture
/// de la fenêtre si elle est encore à venir : le troisième jour du séjour, le marché de chaque
/// matin est celui de demain, pas celui du premier jour.
pub fn events_within(
    events: &[EventRow],
    window: Option<(DateTime<Utc>, DateTime<Utc>)>,
    tz: Option<&PropertyTz>,
    now: DateTime<Utc>,
) -> Vec<EventRow> {
    let Some((from, to)) = window else {
        return events.to_vec();
    };
    events
        .iter()
        .map(|event| event.next_from(from.max(now), tz))
        .filter(|event| match parse_starts_at(&event.starts_at) {
            Some(at) => at >= from && at <= to,
            None => true,
        })
        .collect()
}

/// Filters out past events for the home card.
///
/// `now` is passed in from the host context (via `time::now()`); calling `Utc::now()` here would
/// panic in the Wasm sandbox, which has no wall clock.
pub fn events_for_home_card(
    events: &[EventRow],
    now: DateTime<Utc>,
    tz: Option<&PropertyTz>,
) -> Vec<EventRow> {
    let parsed: Vec<(EventRow, Option<DateTime<Utc>>)> = events
        .iter()
        .map(|e| e.next_from(now, tz))
        .map(|e| {
            let at = parse_starts_at(&e.starts_at);
            (e, at)
        })
        .collect();
    let any_parseable = parsed.iter().any(|(_, at)| at.is_some());
    if !any_parseable {
        return events.to_vec();
    }
    parsed
        .into_iter()
        .filter_map(|(event, at)| {
            let at = at?;
            if at >= now {
                Some(event)
            } else {
                None
            }
        })
        .collect()
}

pub fn sort_events_by_start(mut events: Vec<EventRow>) -> Vec<EventRow> {
    events.sort_by(|left, right| {
        let left_at = parse_starts_at(&left.starts_at);
        let right_at = parse_starts_at(&right.starts_at);
        match (left_at, right_at) {
            (Some(l), Some(r)) => l.cmp(&r),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => left.id.cmp(&right.id),
        }
    });
    events
}

/// Le badge « quand » d'un événement : annulé d'abord, puis toute la journée, sinon l'heure —
/// précédés du jour quand ce n'est pas aujourd'hui : « Demain à 20:30 », « Samedi · Toute la
/// journée ». Aujourd'hui, c'est le badge « Ce soir » / « Aujourd'hui » qui le dit.
///
/// Sans jour, un événement de la semaine du séjour ne se distinguait pas de celui du soir même.
pub fn event_when(
    event: &EventRow,
    tz: Option<&PropertyTz>,
    now: DateTime<Utc>,
    locale: &str,
) -> String {
    if event.cancelled {
        return "i18n:guest.event.cancelled".to_string();
    }
    let day = parse_starts_at(&event.starts_at).and_then(|at| day_label(at, now, tz, locale));
    match (day, event.all_day) {
        (None, true) => "i18n:guest.event.allDay".to_string(),
        (None, false) => format_starts_at_display(&event.starts_at, tz),
        (Some(day), true) => {
            let all_day = t!("guest.event.allDay").unwrap_or_else(|_| "guest.event.allDay".into());
            format!("{day} · {all_day}")
        }
        (Some(day), false) => format!("{day} {}", format_starts_at_display(&event.starts_at, tz)),
    }
}

/// « Demain », « Samedi », « Hier », « 12 juin » : le jour de `at` vu de `now`, à l'heure du logement.
/// `None` pour aujourd'hui, que le badge du jour annonce.
pub fn day_label(
    at: DateTime<Utc>,
    now: DateTime<Utc>,
    tz: Option<&PropertyTz>,
    locale: &str,
) -> Option<String> {
    let key = match relative_day(crate::config::local(at, tz), crate::config::local(now, tz)) {
        RelativeDay::Today | RelativeDay::Tonight => return None,
        RelativeDay::Date(date) => return Some(short_date(date, locale)),
        RelativeDay::Tomorrow => "tomorrow",
        RelativeDay::Yesterday => "yesterday",
        RelativeDay::Weekday(day) => crate::config::WEEKDAYS[day.num_days_from_monday() as usize],
        // Un cas que le SDK ajouterait : la date, qui reste juste.
        _ => return Some(short_date(crate::config::local(at, tz).date(), locale)),
    };
    let key = format!("guest.day.{key}");
    Some(t!(&key).unwrap_or(key))
}

/// La clé du badge d'un événement du jour : « Ce soir » à partir de 18 h, « Aujourd'hui » avant.
pub fn today_key(event: &EventRow, tz: Option<&PropertyTz>) -> &'static str {
    let evening = parse_starts_at(&event.starts_at).is_some_and(|at| {
        crate::config::local(at, tz).hour() >= portaki_sdk::host::time::EVENING_FROM_HOUR
    });
    if evening {
        "guest.event.tonight"
    } else {
        "guest.day.today"
    }
}

/// Le badge du jour (§9 #2) : l'événement commence aujourd'hui, à l'heure du logement — pas celle
/// du serveur : 22 h 30 UTC, c'est déjà demain à Paris. Sans fuseau connu, pas de badge plutôt
/// qu'un « ce soir » deviné. Ni annulé, ni toute la journée, ni déjà fini (sa fin, sinon son
/// début, passé) : le badge annonce un moment à venir — « Ce soir », ou « Aujourd'hui » avant
/// 18 h ([`today_key`]).
pub fn is_tonight(
    event: &EventRow,
    now: DateTime<Utc>,
    tz: Option<&portaki_sdk::host::time::PropertyTz>,
) -> bool {
    let (Some(tz), Some(starts)) = (tz, parse_starts_at(&event.starts_at)) else {
        return false;
    };
    let ends = event
        .ends_at
        .as_deref()
        .and_then(parse_starts_at)
        .unwrap_or(starts);
    !event.cancelled
        && !event.all_day
        && ends >= now
        && tz.to_local(starts).date_naive() == tz.to_local(now).date_naive()
}

/// « Tous les mardis et jeudis » : les jours d'un événement hebdomadaire, à l'heure du logement.
/// `None` pour un événement qui ne se répète pas chaque semaine, ou sans jour lisible.
pub fn weekly_line(event: &EventRow, tz: Option<&PropertyTz>) -> Option<String> {
    if event.recurrence() != "weekly" {
        return None;
    }
    let days: Vec<String> = event
        .effective_weekdays(tz)
        .into_iter()
        .map(|day| {
            let key = crate::config::WEEKDAYS[day.num_days_from_monday() as usize];
            t!(&format!("guest.recurrence.day.{key}")).unwrap_or_else(|_| key.to_string())
        })
        .collect();
    let (last, rest) = days.split_last()?;
    let days = if rest.is_empty() {
        last.clone()
    } else {
        let separator = t!("guest.recurrence.separator").unwrap_or_else(|_| ", ".into());
        let and = t!("guest.recurrence.and").unwrap_or_else(|_| " & ".into());
        format!("{}{and}{last}", rest.join(&separator))
    };
    Some(t!("guest.recurrence.weekly", days = &days).unwrap_or(days))
}

pub fn format_starts_at_display(raw: &str, tz: Option<&PropertyTz>) -> String {
    let Some(at) = parse_starts_at(raw) else {
        return raw.trim().to_string();
    };
    let time = local_time(at, tz);
    t!("guest.event.startsAt", time = &time).unwrap_or_else(|_| time)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn home_card_filters_past_when_parseable() {
        let past = Utc.with_ymd_and_hms(2020, 1, 1, 12, 0, 0).unwrap();
        let future = Utc.with_ymd_and_hms(2099, 6, 1, 18, 0, 0).unwrap();
        let events = vec![
            EventRow {
                id: "a".into(),
                title: Default::default(),
                place: Default::default(),
                starts_at: past.to_rfc3339(),
                ends_at: None,
                url: None,
                lat: None,
                lng: None,
                note: None,
                ..Default::default()
            },
            EventRow {
                id: "b".into(),
                title: Default::default(),
                place: Default::default(),
                starts_at: future.to_rfc3339(),
                ends_at: None,
                url: None,
                lat: None,
                lng: None,
                note: None,
                ..Default::default()
            },
        ];
        let now = Utc.with_ymd_and_hms(2050, 1, 1, 0, 0, 0).unwrap();
        let filtered = events_for_home_card(&events, now, None);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "b");
    }

    /// §0.7 : de la veille de l'arrivée au lendemain du départ, et rien au-delà.
    #[test]
    fn the_window_opens_the_day_before_and_closes_the_day_after() {
        let stay = StayContext {
            checkin_at: Some(at("2026-08-10T15:00:00Z")),
            checkout_at: Some(at("2026-08-13T10:00:00Z")),
            ..StayContext::default()
        };
        let (from, to) = stay_window(Some(&stay)).expect("fenêtre");
        assert_eq!(from, at("2026-08-09T15:00:00Z"));
        assert_eq!(to, at("2026-08-14T10:00:00Z"));

        let events = [
            row("avant", "2026-08-01T20:00:00Z"),
            row("la veille", "2026-08-09T20:00:00Z"),
            row("pendant", "2026-08-11T20:00:00Z"),
            row("le lendemain", "2026-08-14T09:00:00Z"),
            row("le mois prochain", "2026-09-15T20:00:00Z"),
            row("sans date", ""),
        ];
        let kept: Vec<String> = events_within(&events, Some((from, to)), None, from)
            .into_iter()
            .map(|e| e.title.get("fr").to_string())
            .collect();
        assert_eq!(kept, ["la veille", "pendant", "le lendemain", "sans date"]);
    }

    /// Sans dates de séjour, rien n'est écarté : mieux vaut la liste entière qu'une fenêtre
    /// inventée autour d'une date qu'on n'a pas.
    #[test]
    fn no_stay_no_window() {
        assert_eq!(stay_window(None), None);
        let events = [row("un jour", "2027-01-01T20:00:00Z")];
        assert_eq!(
            events_within(&events, None, None, at("2026-08-01T00:00:00Z")).len(),
            1
        );
    }

    /// Le jour est celui du logement : 22 h 30 UTC le 14, c'est 00 h 30 le 15 à Paris.
    #[test]
    fn tonight_is_today_at_the_property() {
        use portaki_sdk::host::time::PropertyTz;
        let now = at("2026-07-14T21:30:00Z");
        let late = row("feu", "2026-07-14T22:30:00Z");
        let paris = PropertyTz::parse("Europe/Paris");
        let utc = PropertyTz::parse("UTC");
        assert!(!is_tonight(&late, now, paris.as_ref()));
        assert!(is_tonight(&late, now, utc.as_ref()));
        assert!(!is_tonight(&late, now, None));
        let evening = row("concert", "2026-07-14T21:45:00Z");
        assert!(is_tonight(&evening, now, paris.as_ref()));
        let cancelled = EventRow {
            cancelled: true,
            ..evening.clone()
        };
        assert!(!is_tonight(&cancelled, now, paris.as_ref()));
        // Fini à 10 h : plus « ce soir » à 23 h 30.
        let morning = EventRow {
            ends_at: Some("2026-07-14T10:00:00Z".into()),
            ..row("marché", "2026-07-14T08:00:00Z")
        };
        assert!(!is_tonight(&morning, now, paris.as_ref()));
        assert!(!is_tonight(
            &row("demain", "2026-07-15T19:00:00Z"),
            now,
            paris.as_ref()
        ));
    }

    /// L'heure affichée est celle du logement : l'heure du mur des sélecteurs ressort telle
    /// quelle, et un instant UTC d'une ancienne ligne est ramené à Paris (UTC+2 l'été) comme à
    /// New York (UTC−4).
    #[test]
    #[serial_test::serial]
    fn times_show_at_the_property() {
        let mut mock = portaki_test_utils::MockContext::guest();
        let bundle: std::collections::BTreeMap<String, String> =
            serde_json::from_str(include_str!("../i18n/fr-FR.json")).unwrap();
        for (key, value) in bundle {
            mock = mock.with_translation(key, value);
        }
        mock.run(|_| {
            let wall = EventRow {
                ends_at: Some("2026-08-15T23:15:00".into()),
                ..row("feu", "2026-08-15T22:30:00")
            };
            let instant = row("marché", "2026-08-15T20:30:00Z");
            let shown = |event: &EventRow, tz: &str| {
                let tz = PropertyTz::parse(tz);
                let event = anchored(event.clone(), tz.as_ref());
                let starts = local_time(at(&event.starts_at), tz.as_ref());
                (
                    event_when(&event, tz.as_ref(), at(&event.starts_at), "fr-FR"),
                    starts,
                    event,
                )
            };
            let when = |time: &str| {
                t!("guest.event.startsAt", time = time).unwrap_or_else(|_| time.to_string())
            };

            let (paris_when, paris_time, paris) = shown(&wall, "Europe/Paris");
            assert_eq!((paris_when, paris_time.as_str()), (when("22:30"), "22:30"));
            assert_eq!(at(&paris.starts_at), at("2026-08-15T20:30:00Z"));
            assert_eq!(
                paris.ends_at.as_deref().map(at),
                Some(at("2026-08-15T21:15:00Z"))
            );
            let (ny_when, ny_time, ny) = shown(&wall, "America/New_York");
            assert_eq!((ny_when, ny_time.as_str()), (when("22:30"), "22:30"));
            assert_eq!(at(&ny.starts_at), at("2026-08-16T02:30:00Z"));

            assert_eq!(shown(&instant, "Europe/Paris").1, "22:30");
            assert_eq!(shown(&instant, "America/New_York").1, "16:30");
            // Sans fuseau connu, rien ne bouge : UTC, comme avant.
            assert_eq!(shown(&wall, "").1, "22:30");
            assert_eq!(shown(&instant, "").1, "20:30");
        });
    }

    /// 22 h 30 saisi au formulaire, c'est ce soir à Paris dès le 15 à 22 h (20 h UTC) — lu en
    /// UTC, il tombait à 0 h 30 le 16 et perdait le badge. À New York, ce soir le 15 à 16 h 30,
    /// pas encore la veille à 23 h.
    #[test]
    fn a_wall_clock_event_is_tonight_at_the_property() {
        let event = row("feu", "2026-08-15T22:30:00");
        let paris = PropertyTz::parse("Europe/Paris");
        let event_paris = anchored(event.clone(), paris.as_ref());
        assert!(is_tonight(
            &event_paris,
            at("2026-08-15T20:00:00Z"),
            paris.as_ref()
        ));
        // 23 h à Paris : commencé depuis une demi-heure, sans fin donnée, plus de badge.
        assert!(!is_tonight(
            &event_paris,
            at("2026-08-15T21:00:00Z"),
            paris.as_ref()
        ));
        let ny = PropertyTz::parse("America/New_York");
        let event_ny = anchored(event, ny.as_ref());
        assert!(is_tonight(
            &event_ny,
            at("2026-08-15T20:30:00Z"),
            ny.as_ref()
        ));
        assert!(!is_tonight(
            &event_ny,
            at("2026-08-15T03:00:00Z"),
            ny.as_ref()
        ));
    }

    fn at(raw: &str) -> DateTime<Utc> {
        parse_starts_at(raw).expect("date")
    }

    fn row(title: &str, starts_at: &str) -> EventRow {
        EventRow {
            title: portaki_sdk::contracts::i18n::I18nText::new(title, title),
            starts_at: starts_at.to_string(),
            ..EventRow::default()
        }
    }

    /// « Tous les mardis et jeudis » : les jours au pluriel, joints par la langue, et rien pour
    /// ce qui ne se répète pas chaque semaine.
    #[test]
    #[serial_test::serial]
    fn the_weekly_line_names_the_days() {
        let bundle: std::collections::BTreeMap<String, String> =
            serde_json::from_str(include_str!("../i18n/fr-FR.json")).unwrap();
        let mut mock = portaki_test_utils::MockContext::guest();
        for (key, value) in bundle {
            mock = mock.with_translation(key, value);
        }
        mock.run(|_| {
            let weekly = |days: &[&str]| EventRow {
                starts_at: "2026-01-06T20:00:00Z".into(),
                recurrence: "weekly".into(),
                weekdays: days.iter().map(|d| d.to_string()).collect(),
                ..EventRow::default()
            };
            assert_eq!(
                weekly_line(&weekly(&["thu", "tue"]), None).as_deref(),
                Some("Tous les mardis et jeudis")
            );
            assert_eq!(
                weekly_line(&weekly(&["mon", "wed", "fri"]), None).as_deref(),
                Some("Tous les lundis, mercredis et vendredis")
            );
            // Sans jour choisi : celui de la date de début, un mardi.
            assert_eq!(
                weekly_line(&weekly(&[]), None).as_deref(),
                Some("Tous les mardis")
            );
            let monthly = EventRow {
                recurrence: "monthly".into(),
                ..weekly(&["tue"])
            };
            assert_eq!(weekly_line(&monthly, None), None);
        });
    }
}
