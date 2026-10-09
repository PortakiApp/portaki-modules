//! Host configuration, held by the platform (`#[portaki_sdk::config]`).
//!
//! The keys are the names of the host form fields: the platform takes `updateConfig` itself. The
//! form may send the radius and the event coordinates as text (`radius_km: "15"`, `lat: "43.5"`);
//! the readers below accept that and numbers alike.

use portaki_sdk::config::check;
use portaki_sdk::contracts::i18n::I18nText;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// Le rayon d'un hôte qui n'en a jamais choisi (§2.1). Un rayon déjà enregistré n'est jamais
/// réécrit : hors de `MIN..=MAX`, il est signalé sous le champ (sans bloquer la publication) et
/// borné à la lecture.
const DEFAULT_RADIUS_KM: u32 = 15;
const MIN_RADIUS_KM: u32 = 1;
const MAX_RADIUS_KM: u32 = 50;
/// Un conseil, une ligne : 120 caractères au plus (§2.2).
const TIP_MAX: usize = 120;

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

    /// Le message à afficher sous `field`, s'il y en a un.
    pub fn error_of(&self, field: &str) -> Option<I18nText> {
        self.problems()
            .into_iter()
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
    /// `none`, `weekly` ou `monthly` ([`RECURRENCES`]) : chaque semaine le jour de la date de
    /// début, chaque mois à la même date.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub recurrence: String,
    /// Annulé : badge « Annulé », plus de bouton de réservation (§2.2).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub cancelled: bool,
}

/// Les répétitions proposées (§2.2). « Dates choisies » attend une liste par événement.
pub const RECURRENCES: [&str; 3] = ["none", "weekly", "monthly"];
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

    /// L'événement à sa prochaine occurrence à partir de `from` : la date de début (et de fin)
    /// avancée d'une semaine ou d'un mois tant qu'elle précède `from`. Inchangé sans répétition,
    /// sans date lisible, ou quand il commence déjà après `from`.
    pub fn next_from(&self, from: chrono::DateTime<chrono::Utc>) -> EventRow {
        use chrono::Months;
        let Some(starts) = crate::time_format::parse_starts_at(&self.starts_at) else {
            return self.clone();
        };
        let step = |at: chrono::DateTime<chrono::Utc>, n: u32| match self.recurrence() {
            "weekly" => Some(at + chrono::Duration::weeks(i64::from(n))),
            "monthly" => at.checked_add_months(Months::new(n)),
            _ => None,
        };
        // Le nombre de pas, d'un coup : un événement hebdomadaire saisi il y a trois ans ne doit
        // pas boucler mille fois.
        let steps = match self.recurrence() {
            "weekly" if starts < from => ((from - starts).num_days() as u32).div_ceil(7),
            "monthly" if starts < from => {
                use chrono::Datelike;
                let months = (from.year() - starts.year()) * 12 + from.month() as i32
                    - starts.month() as i32;
                months.max(0) as u32
            }
            _ => 0,
        };
        let mut next = starts;
        let mut n = steps;
        while let Some(at) = step(starts, n) {
            next = at;
            if at >= from {
                break;
            }
            n += 1;
        }
        if next == starts {
            return self.clone();
        }
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
        let next = event.next_from(from);
        assert_eq!(next.starts_at, "2026-07-14T08:00:00+00:00");
        assert_eq!(next.ends_at.as_deref(), Some("2026-07-14T12:00:00+00:00"));
        let monthly = EventRow {
            recurrence: "monthly".into(),
            ..event.clone()
        };
        assert_eq!(
            monthly.next_from(from).starts_at,
            "2026-08-06T08:00:00+00:00"
        );
        // Sans répétition, rien ne bouge.
        let once = EventRow {
            recurrence: String::new(),
            ..event
        };
        assert_eq!(once.next_from(from).starts_at, "2026-01-06T08:00:00Z");
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
}
