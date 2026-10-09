//! Host configuration, held by the platform (`#[portaki_sdk::config]`).
//!
//! The keys are the names of the host form fields: the platform takes `updateConfig` itself. The
//! form may send the radius and the event coordinates as text (`radius_km: "15"`, `lat: "43.5"`);
//! the readers below accept that and numbers alike.

use portaki_sdk::config::check;
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::host::time::PropertyTz;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// Le rayon d'un hôte qui n'en a jamais choisi (§2.1). Un rayon déjà enregistré n'est jamais
/// réécrit : hors de `MIN..=MAX`, il est signalé sous le champ (sans bloquer la publication) et
/// borné à la lecture.
const DEFAULT_RADIUS_KM: u32 = 15;
const MIN_RADIUS_KM: u32 = 1;
const MAX_RADIUS_KM: u32 = 50;
/// Un conseil, une ligne : 120 caractères au plus, et cinq conseils au plus (§2.2).
const TIP_MAX: usize = 120;
const TIPS_MAX: usize = 5;

#[portaki_sdk::config]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleConfig {
    /// The rows of the host form, blank ones included (see [`Self::parse_events`]).
    #[field(structured, recommended, label = "config.events")]
    pub events: Vec<EventRow>,
    #[field(label = "host.disclaimer.label")]
    pub disclaimer: I18nText,
    /// When true, guest surfaces also load OpenAgenda events around the property.
    #[field(label = "host.nearby.enabled")]
    pub nearby_enabled: bool,
    /// Search radius in kilometers (bbox approximation for OpenAgenda `geo`).
    #[field(kind = "number", label = "host.nearby.radius")]
    #[serde(deserialize_with = "deserialize_radius")]
    pub radius_km: u32,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            disclaimer: I18nText::default(),
            nearby_enabled: true,
            radius_km: DEFAULT_RADIUS_KM,
        }
    }
}

impl ModuleConfig {
    pub fn is_empty(&self) -> bool {
        self.parse_events().is_empty() && !self.nearby_enabled && self.disclaimer.is_blank()
    }

    /// The filled rows, for the guest, in form order. A row without an id takes its slot's
    /// (`evt-1`…), as the module's own `updateConfig` used to give it.
    pub fn parse_events(&self) -> Vec<EventRow> {
        self.events
            .iter()
            .enumerate()
            .filter(|(_, e)| !e.title.is_blank())
            .map(|(index, e)| {
                let mut e = e.clone();
                if e.id.trim().is_empty() {
                    e.id = format!("evt-{}", index + 1);
                }
                e
            })
            .collect()
    }

    pub fn normalized_radius_km(&self) -> u32 {
        self.radius_km.clamp(MIN_RADIUS_KM, MAX_RADIUS_KM)
    }

    /// Ce qui ne va pas, champ par champ (`events.<i>.title`…) — sous le champ dans le formulaire,
    /// et dans `publishReadiness`. Le lieu sans position n'y est pas : il avertit, il ne bloque pas.
    pub fn problems(&self) -> Vec<(String, I18nText)> {
        let text = crate::i18n::text;
        let too_long = |value: &I18nText, max: usize| {
            value
                .by_language()
                .find_map(|(_, text)| check::max_chars(text, max))
        };
        let mut problems: Vec<(String, I18nText)> = Vec::new();
        let filled = self.events.iter().filter(|e| !e.is_blank()).count();
        if !(MIN_RADIUS_KM..=MAX_RADIUS_KM).contains(&self.radius_km) {
            problems.push(("radius_km".into(), text("host.nearby.radius.range")));
        }
        if filled > MAX_EVENTS {
            problems.push(("events".into(), text("host.events.tooMany")));
        }
        for (index, event) in self.events.iter().enumerate() {
            if event.is_blank() {
                continue;
            }
            let mut push = |key: &str, error: Option<I18nText>| {
                if let Some(error) = error {
                    problems.push((format!("events.{index}.{key}"), error));
                }
            };
            push(
                "title",
                if event.title.is_blank() {
                    Some(text("host.event.title.required"))
                } else {
                    too_long(&event.title, TITLE_MAX)
                },
            );
            let ends_before = event
                .ends_at
                .as_deref()
                .and_then(crate::time_format::parse_starts_at)
                .zip(crate::time_format::parse_starts_at(&event.starts_at))
                .is_some_and(|(ends, starts)| ends < starts);
            push(
                "ends_at",
                ends_before.then(|| text("host.event.ends.beforeStart")),
            );
            push("url", event.url.as_deref().and_then(check::https_url));
            push(
                "price",
                event
                    .price
                    .as_deref()
                    .and_then(|price| check::max_chars(price, PRICE_MAX)),
            );
            push("access", too_long(&event.access, ACCESS_MAX));
            push(
                "tips",
                event.tips.by_language().find_map(|(_, tips)| {
                    tips.lines().find_map(|tip| check::max_chars(tip, TIP_MAX))
                }),
            );
            push(
                "note",
                event
                    .note
                    .as_ref()
                    .and_then(|note| too_long(note, NOTE_MAX)),
            );
        }
        problems
    }

    /// Ce qui mérite un coup d'œil sans bloquer la publication (§2.2) : des contrôles venus
    /// après les données, qu'un hôte déjà publié ne doit pas découvrir en se voyant refuser.
    pub fn warnings(&self) -> Vec<(String, I18nText)> {
        let text = crate::i18n::text;
        let mut warnings = Vec::new();
        for (index, event) in self.events.iter().enumerate() {
            if event.is_blank() {
                continue;
            }
            match event.recurrence() {
                "weekly" if event.weekdays.is_empty() => warnings.push((
                    format!("events.{index}.weekdays"),
                    text("host.event.weekdays.required"),
                )),
                "dates" if event.chosen_dates().is_empty() => warnings.push((
                    format!("events.{index}.dates"),
                    text("host.event.weekdays.required"),
                )),
                _ => {}
            }
            let too_many_tips = event
                .tips
                .by_language()
                .any(|(_, tips)| tips.lines().filter(|l| !l.trim().is_empty()).count() > TIPS_MAX);
            if too_many_tips {
                warnings.push((
                    format!("events.{index}.tips"),
                    text("host.event.tips.tooMany"),
                ));
            }
        }
        warnings
    }

    /// Le message à afficher sous `field`, s'il y en a un : l'erreur d'abord, sinon l'avertissement.
    pub fn error_of(&self, field: &str) -> Option<I18nText> {
        self.problems()
            .into_iter()
            .chain(self.warnings())
            .find(|(name, _)| name == field)
            .map(|(_, error)| error)
    }
}

/// An event. The form sends `title`, `place`, `starts_at`, `url`, `lat`, `lng` (and `id`); the
/// platform keeps the rest — `ends_at`, `note`, the texts' other languages.
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct EventRow {
    pub id: String,
    pub title: I18nText,
    pub place: I18nText,
    pub starts_at: String,
    pub ends_at: Option<String>,
    #[serde(deserialize_with = "deserialize_nonempty")]
    pub url: Option<String>,
    #[serde(deserialize_with = "deserialize_coord")]
    pub lat: Option<f64>,
    #[serde(deserialize_with = "deserialize_coord")]
    pub lng: Option<f64>,
    pub note: Option<I18nText>,
    /// L'adresse telle que le sélecteur de carte l'a géocodée, pour la ligne de lieu de la fiche.
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub address: Option<String>,
    /// Ce que ça coûte, écrit par l'hôte : « Gratuit », « 12 € », « dès 8 € ». Pas un nombre :
    /// « Gratuit » n'en est pas un, et c'est la valeur la plus fréquente (§2.11).
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub price: Option<String>,
    /// La photo que l'hôte a déposée, en référence `portaki-file:` — la plateforme l'échange
    /// contre une URL signée au rendu. Vide quand il n'en a pas mis, et la fiche s'ouvre alors
    /// sur son titre.
    #[serde(default)]
    pub photo: String,
    /// Comment on y entre, et si on y entre : « Entrée côté rue, rampe d'accès », « Tram T2
    /// arrêt Palais », « Pas d'ascenseur » (§2.11).
    ///
    /// Une ligne par information, comme les conseils, mais séparé d'eux : un conseil se lit si on
    /// a le temps, l'accès se lit avant de partir — et un voyageur en fauteuil le cherche en
    /// premier, pas au milieu des bons plans de parking.
    #[serde(default)]
    pub access: I18nText,
    /// « Bon à savoir » : une ligne par conseil, écrites par l'hôte.
    #[serde(default)]
    pub tips: I18nText,
    /// Toute la journée : pas d'heure affichée (§2.2).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub all_day: bool,
    /// `none`, `weekly`, `monthly` ou `dates` ([`RECURRENCES`]) : chaque semaine les jours de
    /// [`Self::weekdays`], chaque mois à la même date, ou aux dates de [`Self::dates`].
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub recurrence: String,
    /// Les jours d'un événement hebdomadaire, `mon` … `sun` : un choix multiple. Vide, c'est le
    /// jour de la date de début, comme avant que le choix existe.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_weekdays"
    )]
    pub weekdays: Vec<String>,
    /// Les dates d'un événement « Dates choisies », en plus de la date de début et à son heure.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_dates"
    )]
    pub dates: Vec<EventDate>,
    /// Annulé : badge « Annulé », plus de bouton de réservation (§2.2).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub cancelled: bool,
}

/// Une date choisie, `AAAA-MM-JJ` — une ligne de liste du formulaire (`events.0.dates.1.date`).
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct EventDate {
    pub date: String,
}

/// Les répétitions proposées (§2.2).
pub const RECURRENCES: [&str; 4] = ["none", "weekly", "monthly", "dates"];
/// Les jours de la semaine, dans l'ordre du calendrier.
pub const WEEKDAYS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
/// Combien d'événements la configuration accepte (§2.2).
pub const MAX_EVENTS: usize = 50;
const TITLE_MAX: usize = 80;
const PRICE_MAX: usize = 30;
const ACCESS_MAX: usize = 120;
const NOTE_MAX: usize = 1200;

impl EventRow {
    pub fn has_coords(&self) -> bool {
        match (self.lat, self.lng) {
            (Some(lat), Some(lng)) => lat != 0.0 || lng != 0.0,
            _ => false,
        }
    }

    /// L'identifiant de route de cet événement, pour `events/:eventId`.
    ///
    /// Le rang en secours : un hôte peut avoir des lignes sans id (une config d'avant les slots,
    /// un événement venu de la recherche à proximité), et deux fiches ne doivent pas partager
    /// une adresse.
    pub fn route_id(&self, index: usize) -> String {
        let id = self.id.trim();
        if id.is_empty() {
            format!("e{index}")
        } else {
            id.to_string()
        }
    }

    /// La durée en minutes, quand l'hôte a donné une fin postérieure au début.
    pub fn duration_minutes(&self) -> Option<i64> {
        let starts = crate::time_format::parse_starts_at(&self.starts_at)?;
        let ends = crate::time_format::parse_starts_at(self.ends_at.as_deref()?)?;
        let minutes = (ends - starts).num_minutes();
        (minutes > 0).then_some(minutes)
    }

    /// La répétition, dans la liste ; aucune sans choix.
    pub fn recurrence(&self) -> &'static str {
        RECURRENCES
            .iter()
            .find(|key| **key == self.recurrence.trim())
            .unwrap_or(&RECURRENCES[0])
    }

    /// Les jours où l'événement hebdomadaire a lieu, dans l'ordre du calendrier : ceux choisis,
    /// sinon celui de la date de début, à l'heure du logement.
    pub fn effective_weekdays(&self, tz: Option<&PropertyTz>) -> Vec<chrono::Weekday> {
        use chrono::Datelike;
        let chosen: Vec<chrono::Weekday> = WEEKDAYS
            .iter()
            .zip(0u8..)
            .filter(|(key, _)| self.weekdays.iter().any(|day| day == *key))
            .filter_map(|(_, n)| chrono::Weekday::try_from(n).ok())
            .collect();
        if !chosen.is_empty() {
            return chosen;
        }
        crate::time_format::parse_starts_at(&self.starts_at)
            .map(|starts| vec![local(starts, tz).weekday()])
            .unwrap_or_default()
    }

    /// Les dates choisies lisibles, triées.
    pub fn chosen_dates(&self) -> Vec<chrono::NaiveDate> {
        let mut dates: Vec<chrono::NaiveDate> = self
            .dates
            .iter()
            .filter_map(|d| chrono::NaiveDate::parse_from_str(d.date.trim(), "%Y-%m-%d").ok())
            .collect();
        dates.sort();
        dates.dedup();
        dates
    }

    /// L'événement à sa prochaine occurrence à partir de `from` : la date de début (et la fin,
    /// du même écart) portée au premier jour choisi, au mois suivant ou à la date choisie qui ne
    /// précède pas `from`. Les jours et les dates se lisent à l'heure du logement (`tz`, UTC sans
    /// fuseau connu) : 20 h reste 20 h au changement d'heure, et 23 h 30 à Paris n'est pas le
    /// lendemain. Inchangé sans répétition, sans date lisible, ou quand il commence déjà après
    /// `from`.
    pub fn next_from(
        &self,
        from: chrono::DateTime<chrono::Utc>,
        tz: Option<&PropertyTz>,
    ) -> EventRow {
        use chrono::{Datelike, Months};
        let Some(starts) = crate::time_format::parse_starts_at(&self.starts_at) else {
            return self.clone();
        };
        let start_local = local(starts, tz);
        let at_local = |date: chrono::NaiveDate| utc(date.and_time(start_local.time()), tz);
        let next = match self.recurrence() {
            "weekly" => {
                let days = self.effective_weekdays(tz);
                // Au plus huit jours à parcourir à partir du plus tardif des deux : un événement
                // saisi il y a trois ans ne boucle pas mille fois.
                let first = start_local
                    .date()
                    .max(local(from, tz).date() - chrono::Duration::days(1));
                first
                    .iter_days()
                    .take(9)
                    .filter(|date| days.contains(&date.weekday()))
                    .map(at_local)
                    .find(|at| *at >= starts && *at >= from)
            }
            "monthly" => {
                let months = if starts < from {
                    ((from.year() - starts.year()) * 12 + from.month() as i32
                        - starts.month() as i32)
                        .max(0) as u32
                } else {
                    0
                };
                (months..months + 2)
                    .filter_map(|n| starts.checked_add_months(Months::new(n)))
                    .find(|at| *at >= from)
            }
            // La date de début et les dates choisies : la première à venir ; toutes passées, la
            // dernière, que la fenêtre écarte.
            "dates" => {
                let mut dates = self.chosen_dates();
                dates.push(start_local.date());
                dates.sort();
                dates
                    .iter()
                    .map(|date| at_local(*date))
                    .find(|at| *at >= from)
                    .or_else(|| dates.last().map(|date| at_local(*date)))
            }
            _ => None,
        };
        let Some(next) = next.filter(|next| *next != starts) else {
            return self.clone();
        };
        let shift = next - starts;
        let mut event = self.clone();
        event.starts_at = next.to_rfc3339();
        event.ends_at = self
            .ends_at
            .as_deref()
            .and_then(crate::time_format::parse_starts_at)
            .map(|ends| (ends + shift).to_rfc3339());
        event
    }

    /// La photo déposée, en référence, ou `None` quand il n'y en a pas.
    pub fn photo_ref(&self) -> Option<&str> {
        let photo = self.photo.trim();
        (!photo.is_empty()).then_some(photo)
    }

    /// Nothing the form shows: a slot the host left (or emptied).
    pub fn is_blank(&self) -> bool {
        self.title.is_blank()
            && self.place.is_blank()
            && self.starts_at.trim().is_empty()
            && self.url.is_none()
            && self.lat.is_none()
            && self.lng.is_none()
    }
}

/// L'heure du logement, sans fuseau (UTC sans fuseau connu).
fn local(at: chrono::DateTime<chrono::Utc>, tz: Option<&PropertyTz>) -> chrono::NaiveDateTime {
    tz.map_or(at.naive_utc(), |tz| tz.to_local(at).naive_local())
}

/// L'instant d'une heure du logement.
fn utc(at: chrono::NaiveDateTime, tz: Option<&PropertyTz>) -> chrono::DateTime<chrono::Utc> {
    tz.map_or(at.and_utc(), |tz| tz.from_local(at))
}

/// Une liste de jours, en tableau ou en texte : un dashboard d'avant le choix multiple envoie la
/// valeur cochée seule (`"tue"`), ou du JSON en chaîne. Refuser la chaîne ferait refuser toute la
/// configuration. Un jour inconnu est écarté.
fn deserialize_weekdays<'de, D>(deserializer: D) -> std::result::Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: Vec<String> = match Value::deserialize(deserializer)? {
        Value::Array(items) => items
            .into_iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
        Value::String(text) => serde_json::from_str::<Vec<String>>(&text)
            .unwrap_or_else(|_| text.split(',').map(str::to_string).collect()),
        _ => Vec::new(),
    };
    let raw: Vec<String> = raw
        .iter()
        .map(|day| day.trim().to_ascii_lowercase())
        .collect();
    Ok(WEEKDAYS
        .iter()
        .filter(|day| raw.iter().any(|r| r == *day))
        .map(|day| day.to_string())
        .collect())
}

/// Les dates choisies : des lignes `{ "date": "2026-07-14" }` comme le formulaire les envoie, ou
/// des dates nues. Une ligne vide (retirée dans le formulaire) est écartée.
fn deserialize_dates<'de, D>(deserializer: D) -> std::result::Result<Vec<EventDate>, D::Error>
where
    D: Deserializer<'de>,
{
    let Value::Array(items) = Value::deserialize(deserializer)? else {
        return Ok(Vec::new());
    };
    Ok(items
        .into_iter()
        .filter_map(|item| match item {
            Value::String(date) => Some(date),
            Value::Object(row) => row.get("date").and_then(Value::as_str).map(str::to_string),
            _ => None,
        })
        .map(|date| date.trim().to_string())
        .filter(|date| !date.is_empty())
        .map(|date| EventDate { date })
        .collect())
}

/// `15`, `"15"`, or `""` / `null` (the default). A number below zero reads as 0 and one past
/// `u32` as `u32::MAX`: out of range, shown as an error under the field, never a config that no
/// longer loads.
fn deserialize_radius<'de, D>(deserializer: D) -> std::result::Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let saturate = |n: f64| n.round().clamp(0.0, f64::from(u32::MAX)) as u32;
    match Value::deserialize(deserializer)? {
        Value::Null => Ok(DEFAULT_RADIUS_KM),
        Value::Number(n) => n
            .as_f64()
            .map(saturate)
            .ok_or_else(|| serde::de::Error::custom(format!("invalid radius_km {n}"))),
        Value::String(s) if s.trim().is_empty() => Ok(DEFAULT_RADIUS_KM),
        Value::String(s) => s
            .trim()
            .parse::<f64>()
            .map(saturate)
            .map_err(|_| serde::de::Error::custom(format!("invalid radius_km {s:?}"))),
        other => Err(serde::de::Error::custom(format!(
            "invalid radius_km {other}"
        ))),
    }
}

/// `43.5`, `"43.5"`, or `""` / `null` (none). An unparsable text is none too: it was typed in a
/// free text input, and dropping the pin beats refusing the whole config.
fn deserialize_coord<'de, D>(deserializer: D) -> std::result::Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Value::deserialize(deserializer)? {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    })
}

/// A blank text input is no value.
fn deserialize_nonempty<'de, D>(deserializer: D) -> std::result::Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::MockContext;
    use serde_json::json;

    #[test]
    fn disclaimer_plain_string_migrates() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "disclaimer": "Dates indicatives"
        }))
        .unwrap();
        assert_eq!(config.disclaimer.get("fr"), "Dates indicatives");
        assert!(config.nearby_enabled);
        assert_eq!(config.radius_km, DEFAULT_RADIUS_KM);
    }

    /// What the host form sends: the rows, every text a string, the select a string.
    #[test]
    #[serial_test::serial]
    fn the_host_form_shape_reads() {
        let mut slots = vec![
            json!({ "title": "", "place": "", "starts_at": "", "url": "", "lat": "", "lng": "" });
            6
        ];
        slots[1] = json!({
            "title": "Fête du village", "place": "Place centrale", "starts_at": "2099-08-01T20:00:00Z",
            "url": " ", "lat": "43.58", "lng": "7.12"
        });
        let form = json!({
            "events": slots, "disclaimer": "d", "nearby_enabled": false, "radius_km": "20"
        });
        MockContext::host().with_config(&form).run(|ctx| {
            let config = ModuleConfig::load(&ctx).unwrap();
            assert!(config.events[0].is_blank());
            let events = config.parse_events();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].id, "evt-2");
            assert_eq!(events[0].title.get("en"), "Fête du village");
            assert_eq!(events[0].url, None);
            assert_eq!((events[0].lat, events[0].lng), (Some(43.58), Some(7.12)));
            assert_eq!(config.disclaimer.get("fr"), "d");
            assert!(!config.nearby_enabled);
            assert_eq!(config.radius_km, 20);
        });
    }

    /// The old KV blob, read as is: per-language texts, numeric coordinates, `null` for what an
    /// event did not have — and a numeric radius, kept as stored.
    #[test]
    fn the_kv_blob_reads_as_is() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "radius_km": 60,
            "nearby_enabled": false,
            "disclaimer": { "fr": "Dates indicatives", "en": "Dates are indicative" },
            "events": [{
                "id": "evt-1", "title": { "fr": "Concert", "en": "Concert" },
                "place": { "fr": "Plage" }, "starts_at": "2099-07-25T18:00:00Z",
                "ends_at": "2099-07-25T20:00:00Z", "url": null, "lat": 43.5, "lng": 7.1,
                "note": { "fr": "Gratuit", "en": "Free" }
            }, {
                "id": "evt-2", "title": { "fr": "Brocante" }, "place": { "fr": "" },
                "starts_at": "", "ends_at": null, "url": null, "lat": null, "lng": null, "note": null
            }]
        }))
        .unwrap();
        assert_eq!(config.radius_km, 60);
        assert_eq!(config.disclaimer.get("en"), "Dates are indicative");
        assert_eq!(
            config.events[0].ends_at.as_deref(),
            Some("2099-07-25T20:00:00Z")
        );
        assert_eq!(config.events[0].note.as_ref().unwrap().get("en"), "Free");
        assert_eq!(config.events[0].lat, Some(43.5));
        assert_eq!(config.events[1].note, None);
    }

    /// §2.1 : 15 km quand l'hôte n'a rien choisi ; un rayon enregistré hors de 1 à 50 n'est pas
    /// réécrit — il est signalé sous le champ, et borné pour la recherche.
    #[test]
    fn the_radius_defaults_to_15_and_reports_out_of_range() {
        for unset in [
            json!({}),
            json!({ "radius_km": "" }),
            json!({ "radius_km": null }),
        ] {
            let config: ModuleConfig = serde_json::from_value(unset).unwrap();
            assert_eq!(config.radius_km, 15);
            assert!(config.error_of("radius_km").is_none());
        }
        let stored: ModuleConfig = serde_json::from_value(json!({ "radius_km": "60" })).unwrap();
        assert_eq!(stored.radius_km, 60);
        assert_eq!(stored.normalized_radius_km(), 50);
        assert_eq!(
            stored.error_of("radius_km").unwrap().get("fr"),
            "Entre 1 et 50 km."
        );
        let below: ModuleConfig = serde_json::from_value(json!({ "radius_km": -3 })).unwrap();
        assert_eq!(below.normalized_radius_km(), 1);
        assert!(below.error_of("radius_km").is_some());
        let fine: ModuleConfig = serde_json::from_value(json!({ "radius_km": 50 })).unwrap();
        assert!(fine.error_of("radius_km").is_none());
    }

    /// Un conseil par ligne, 120 caractères chacun (§2.2).
    #[test]
    fn a_tip_over_120_characters_is_reported() {
        let long = "x".repeat(121);
        let config: ModuleConfig = serde_json::from_value(json!({
            "events": [
                { "title": "Concert", "tips": format!("{}\n{}", "y".repeat(120), "z".repeat(120)) },
                { "title": "Marché", "tips": { "fr": "Court", "en": long } }
            ]
        }))
        .unwrap();
        let fields: Vec<String> = config.problems().into_iter().map(|(f, _)| f).collect();
        assert_eq!(fields, ["events.1.tips"]);
    }

    #[test]
    #[serial_test::serial]
    fn the_kv_is_read_through_legacy_until_the_platform_holds_the_config() {
        let old = json!({
            "radius_km": 60,
            "disclaimer": { "fr": "Dates indicatives", "en": "Dates are indicative" }
        });
        MockContext::host()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .run(|ctx| {
                let config = ModuleConfig::load(&ctx).unwrap();
                assert_eq!(config.radius_km, 60);
                assert_eq!(config.disclaimer.get("en"), "Dates are indicative");
            });
        MockContext::host()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .with_config(&json!({}))
            .run(|ctx| assert_eq!(ModuleConfig::load(&ctx).unwrap(), ModuleConfig::default()));
    }

    /// Un événement hebdomadaire avance d'une semaine jusqu'à la fenêtre ; sa fin le suit.
    #[test]
    fn a_weekly_event_moves_to_its_next_occurrence() {
        let event = EventRow {
            title: I18nText::from("Marché"),
            starts_at: "2026-01-06T08:00:00Z".into(),
            ends_at: Some("2026-01-06T12:00:00Z".into()),
            recurrence: "weekly".into(),
            ..EventRow::default()
        };
        let from = crate::time_format::parse_starts_at("2026-07-09T00:00:00Z").unwrap();
        let next = event.next_from(from, None);
        assert_eq!(next.starts_at, "2026-07-14T08:00:00+00:00");
        assert_eq!(next.ends_at.as_deref(), Some("2026-07-14T12:00:00+00:00"));
        let monthly = EventRow {
            recurrence: "monthly".into(),
            ..event.clone()
        };
        assert_eq!(
            monthly.next_from(from, None).starts_at,
            "2026-08-06T08:00:00+00:00"
        );
        // Sans répétition, rien ne bouge.
        let once = EventRow {
            recurrence: String::new(),
            ..event
        };
        assert_eq!(once.next_from(from, None).starts_at, "2026-01-06T08:00:00Z");
    }

    /// Les erreurs nomment leur champ ; une ligne vide n'en a pas.
    #[test]
    fn problems_name_their_field() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "events": [
                { "title": "" },
                { "title": "", "place": "Port" },
                { "title": "Concert", "starts_at": "2026-07-14T21:00:00Z",
                  "ends_at": "2026-07-14T20:00:00Z", "url": "http://x.fr" }
            ]
        }))
        .unwrap();
        let fields: Vec<String> = config.problems().into_iter().map(|(f, _)| f).collect();
        assert_eq!(
            fields,
            ["events.1.title", "events.2.ends_at", "events.2.url"]
        );
    }

    fn utc_at(raw: &str) -> chrono::DateTime<chrono::Utc> {
        crate::time_format::parse_starts_at(raw).unwrap()
    }

    fn paris() -> PropertyTz {
        PropertyTz::parse("Europe/Paris").unwrap()
    }

    /// Les jours arrivent en tableau depuis le choix multiple ; un ancien formulaire envoie du
    /// texte. Inconnus écartés, ordre du calendrier quel que soit l'ordre des coches.
    #[test]
    fn weekdays_read_as_array_or_text() {
        for raw in [
            json!(["thu", "TUE", "noel"]),
            json!("[\"thu\",\"tue\"]"),
            json!("thu, tue"),
        ] {
            let event: EventRow = serde_json::from_value(json!({ "weekdays": raw })).unwrap();
            assert_eq!(event.weekdays, ["tue", "thu"]);
        }
        let none: EventRow = serde_json::from_value(json!({ "weekdays": null })).unwrap();
        assert!(none.weekdays.is_empty());
    }

    /// Les dates arrivent en lignes `{ date }` ; une ligne retirée arrive vide et disparaît.
    #[test]
    fn dates_read_as_rows_or_strings() {
        let event: EventRow = serde_json::from_value(json!({
            "dates": [{ "date": "2026-08-14" }, { "date": "" }, "2026-08-02", { "date": "pas une date" }]
        }))
        .unwrap();
        assert_eq!(event.dates.len(), 3);
        let dates: Vec<String> = event.chosen_dates().iter().map(|d| d.to_string()).collect();
        assert_eq!(dates, ["2026-08-02", "2026-08-14"]);
    }

    /// Mardi et jeudi à 20 h : d'une semaine à l'autre, au premier jour choisi à venir.
    #[test]
    fn weekly_with_weekdays_moves_to_the_next_chosen_day() {
        let event = EventRow {
            title: I18nText::from("Marché"),
            // Un lundi.
            starts_at: "2026-01-05T20:00:00Z".into(),
            ends_at: Some("2026-01-05T23:00:00Z".into()),
            recurrence: "weekly".into(),
            weekdays: vec!["tue".into(), "thu".into()],
            ..EventRow::default()
        };
        // Avant le début : le premier jour choisi, pas le lundi de la date de début.
        let early = event.next_from(utc_at("2025-12-01T00:00:00Z"), None);
        assert_eq!(early.starts_at, "2026-01-06T20:00:00+00:00");
        // Un mercredi de juillet : le jeudi.
        let next = event.next_from(utc_at("2026-07-08T12:00:00Z"), None);
        assert_eq!(next.starts_at, "2026-07-09T20:00:00+00:00");
        assert_eq!(next.ends_at.as_deref(), Some("2026-07-09T23:00:00+00:00"));
        // Un jeudi après 20 h : le mardi suivant, une autre semaine, un autre mois.
        let later = event.next_from(utc_at("2026-07-30T21:00:00Z"), None);
        assert_eq!(later.starts_at, "2026-08-04T20:00:00+00:00");
    }

    /// Sans jour choisi, c'est le jour de la date de début — comme avant le choix.
    #[test]
    fn weekly_without_weekdays_keeps_the_start_day() {
        let event = EventRow {
            starts_at: "2026-01-06T08:00:00Z".into(),
            recurrence: "weekly".into(),
            ..EventRow::default()
        };
        let next = event.next_from(utc_at("2026-07-09T00:00:00Z"), None);
        assert_eq!(next.starts_at, "2026-07-14T08:00:00+00:00");
    }

    /// À l'heure du logement : 23 h 30 à Paris un mardi est 21 h 30 UTC, et 20 h reste 20 h
    /// passé le changement d'heure.
    #[test]
    fn weekly_reads_days_and_hours_at_the_property() {
        let tz = paris();
        let late = EventRow {
            // Mardi 6 janvier, 23 h 30 à Paris.
            starts_at: "2026-01-06T22:30:00Z".into(),
            recurrence: "weekly".into(),
            ..EventRow::default()
        };
        assert_eq!(late.effective_weekdays(Some(&tz)), [chrono::Weekday::Tue]);
        assert_eq!(late.effective_weekdays(None), [chrono::Weekday::Tue]);
        // En été, mardi 23 h 30 à Paris, c'est 21 h 30 UTC.
        let summer = late.next_from(utc_at("2026-07-08T00:00:00Z"), Some(&tz));
        assert_eq!(summer.starts_at, "2026-07-14T21:30:00+00:00");

        // Jeudi 20 h à Paris, en hiver (19 h UTC) : après le 29 mars, 18 h UTC.
        let evening = EventRow {
            starts_at: "2026-03-26T19:00:00Z".into(),
            recurrence: "weekly".into(),
            weekdays: vec!["thu".into()],
            ..EventRow::default()
        };
        let after = evening.next_from(utc_at("2026-03-27T00:00:00Z"), Some(&tz));
        assert_eq!(after.starts_at, "2026-04-02T18:00:00+00:00");
    }

    /// Chaque mois, à la même date, d'un mois sur l'autre et d'une année sur l'autre.
    #[test]
    fn monthly_moves_across_months() {
        let event = EventRow {
            starts_at: "2026-01-15T18:00:00Z".into(),
            recurrence: "monthly".into(),
            ..EventRow::default()
        };
        assert_eq!(
            event
                .next_from(utc_at("2026-03-15T18:00:00Z"), None)
                .starts_at,
            "2026-03-15T18:00:00+00:00"
        );
        assert_eq!(
            event
                .next_from(utc_at("2026-03-15T19:00:00Z"), None)
                .starts_at,
            "2026-04-15T18:00:00+00:00"
        );
        assert_eq!(
            event
                .next_from(utc_at("2026-12-20T00:00:00Z"), None)
                .starts_at,
            "2027-01-15T18:00:00+00:00"
        );
    }

    /// Dates choisies : la date de début et la liste, à l'heure du début (au logement) ; toutes
    /// passées, la dernière, que la fenêtre écartera.
    #[test]
    fn chosen_dates_give_the_next_one() {
        let tz = paris();
        let event = EventRow {
            // Vendredi 3 juillet, 21 h à Paris.
            starts_at: "2026-07-03T19:00:00Z".into(),
            ends_at: Some("2026-07-03T21:00:00Z".into()),
            recurrence: "dates".into(),
            dates: vec![
                EventDate {
                    date: "2026-08-14".into(),
                },
                EventDate {
                    date: "2026-07-24".into(),
                },
            ],
            ..EventRow::default()
        };
        let before = event.next_from(utc_at("2026-07-01T00:00:00Z"), Some(&tz));
        assert_eq!(before.starts_at, event.starts_at);
        let next = event.next_from(utc_at("2026-07-10T00:00:00Z"), Some(&tz));
        assert_eq!(next.starts_at, "2026-07-24T19:00:00+00:00");
        assert_eq!(next.ends_at.as_deref(), Some("2026-07-24T21:00:00+00:00"));
        let after = event.next_from(utc_at("2026-07-25T00:00:00Z"), Some(&tz));
        assert_eq!(after.starts_at, "2026-08-14T19:00:00+00:00");
        let past = event.next_from(utc_at("2026-09-01T00:00:00Z"), Some(&tz));
        assert_eq!(past.starts_at, "2026-08-14T19:00:00+00:00");
        // Dans la fenêtre d'un séjour du 20 au 27 juillet : le 24, pas le 3.
        let window = Some((
            utc_at("2026-07-19T00:00:00Z"),
            utc_at("2026-07-28T00:00:00Z"),
        ));
        let kept = crate::time_format::events_within(&[event], window, Some(&tz));
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].starts_at, "2026-07-24T19:00:00+00:00");
    }

    /// Les jours manquants et le sixième conseil avertissent sous le champ, sans compter parmi
    /// les erreurs qui bloquent.
    #[test]
    fn missing_days_and_too_many_tips_only_warn() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "events": [
                { "title": "Marché", "recurrence": "weekly", "starts_at": "2026-07-07T08:00:00Z" },
                { "title": "Festival", "recurrence": "dates", "dates": [{ "date": "" }] },
                { "title": "Concert", "recurrence": "weekly", "weekdays": ["tue"],
                  "tips": "1\n2\n3\n4\n5\n6" },
                { "title": "Brocante", "recurrence": "dates", "dates": ["2026-08-02"],
                  "tips": "1\n2\n3\n4\n5" }
            ]
        }))
        .unwrap();
        assert!(config.problems().is_empty(), "{:?}", config.problems());
        let fields: Vec<String> = config.warnings().into_iter().map(|(f, _)| f).collect();
        assert_eq!(
            fields,
            ["events.0.weekdays", "events.1.dates", "events.2.tips"]
        );
        assert_eq!(
            config.error_of("events.0.weekdays").unwrap().get("fr"),
            "Choisissez au moins un jour."
        );
        assert_eq!(
            config.error_of("events.1.dates").unwrap().get("fr"),
            "Choisissez au moins un jour."
        );
        assert_eq!(
            config.error_of("events.2.tips").unwrap().get("fr"),
            "5 conseils au maximum."
        );
    }

    /// Une configuration enregistrée avant ces champs se relit à l'identique.
    #[test]
    fn a_stored_event_without_the_new_fields_reads() {
        let stored = json!({
            "id": "evt-1", "title": { "fr": "Marché" }, "starts_at": "2026-01-06T08:00:00Z",
            "recurrence": "weekly"
        });
        let event: EventRow = serde_json::from_value(stored).unwrap();
        assert!(event.weekdays.is_empty() && event.dates.is_empty());
        let round: EventRow =
            serde_json::from_value(serde_json::to_value(&event).unwrap()).unwrap();
        assert_eq!(round, event);
    }
}
