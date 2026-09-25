//! Host configuration, held by the platform (`#[portaki_sdk::config]`).
//!
//! The keys are the names of the host form fields: the platform takes `updateConfig` itself. The
//! form sends the select and the event coordinates as text (`radius_km: "40"`, `lat: "43.5"`);
//! the readers below accept that and numbers alike.

use portaki_sdk::contracts::i18n::I18nText;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

const DEFAULT_RADIUS_KM: u32 = 40;
const MIN_RADIUS_KM: u32 = 5;
const MAX_RADIUS_KM: u32 = 100;
const RADIUS_OPTIONS: [u32; 5] = [10, 20, 40, 60, 100];

#[portaki_sdk::config(legacy = legacy)]
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
    #[field(
        kind = "select",
        options = ["10", "20", "40", "60", "100"],
        label = "host.nearby.radius"
    )]
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

/// The old KV blob kept the radius as a number, which the select (text) rejects: it becomes the
/// nearest option. Everything else already has the declared shape.
fn legacy(mut old: Value) -> Value {
    if let Some(radius) = old.get("radius_km").and_then(Value::as_u64) {
        let nearest = RADIUS_OPTIONS
            .into_iter()
            .min_by_key(|option| u64::from(*option).abs_diff(radius))
            .unwrap_or(DEFAULT_RADIUS_KM);
        old["radius_km"] = Value::from(nearest.to_string());
    }
    old
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
}

impl EventRow {
    pub fn has_coords(&self) -> bool {
        match (self.lat, self.lng) {
            (Some(lat), Some(lng)) => lat != 0.0 || lng != 0.0,
            _ => false,
        }
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

/// `40`, `"40"`, or `""` (the default) — the host form sends the select value as a string.
fn deserialize_radius<'de, D>(deserializer: D) -> std::result::Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Number(n) => n
            .as_u64()
            .map(|n| u32::try_from(n).unwrap_or(MAX_RADIUS_KM))
            .ok_or_else(|| serde::de::Error::custom(format!("invalid radius_km {n}"))),
        Value::String(s) if s.trim().is_empty() => Ok(DEFAULT_RADIUS_KM),
        Value::String(s) => s
            .trim()
            .parse()
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

    /// The old KV blob: a numeric radius (the select takes text), a `{fr, en}` disclaimer, rows
    /// with per-language texts, numeric coordinates, `null` for what an event did not have.
    #[test]
    fn legacy_maps_the_kv_blob() {
        let old = json!({
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
        });
        let mapped = legacy(old.clone());
        let mut expected = old;
        expected["radius_km"] = json!("60");
        assert_eq!(mapped, expected);
        let config: ModuleConfig = serde_json::from_value(mapped).unwrap();
        assert_eq!(config.radius_km, 60);
        assert_eq!(config.disclaimer.get("en"), "Dates are indicative");
        assert_eq!(
            config.events[0].ends_at.as_deref(),
            Some("2099-07-25T20:00:00Z")
        );
        assert_eq!(config.events[0].note.as_ref().unwrap().get("en"), "Free");
        assert_eq!(config.events[0].lat, Some(43.5));
        assert_eq!(config.events[1].note, None);
        // A radius off the select: the nearest option, never dropped.
        assert_eq!(legacy(json!({ "radius_km": 25 }))["radius_km"], "20");
        assert_eq!(legacy(json!({ "radius_km": 500 }))["radius_km"], "100");
        assert_eq!(
            legacy(json!({ "disclaimer": "d" })),
            json!({ "disclaimer": "d" })
        );
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
}
