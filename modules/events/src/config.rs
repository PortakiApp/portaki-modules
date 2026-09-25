//! Host configuration, held by the platform (`#[portaki_sdk::config]`).
//!
//! The keys are the names of the host form fields: the platform takes `updateConfig` itself and
//! stores what the form sends — every text as a plain string (`radius_km: "40"`, event
//! `lat: "43.5"`, `title: "Concert"`). The readers below accept both that and the old KV shape
//! (numbers, `{fr, en}` maps).

use std::collections::BTreeMap;

use portaki_sdk::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

const DEFAULT_RADIUS_KM: u32 = 40;
const MIN_RADIUS_KM: u32 = 5;
const MAX_RADIUS_KM: u32 = 100;

#[portaki_sdk::config]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleConfig {
    /// The six manual slots of the host form, empty ones included (see [`Self::parse_events`]).
    #[field(structured, recommended, label = "config.events")]
    pub events: Vec<EventRow>,
    #[field(kind = "textarea", label = "host.disclaimer.label")]
    #[serde(deserialize_with = "deserialize_localized_field")]
    pub disclaimer: Localized,
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
            disclaimer: Localized::default(),
            nearby_enabled: true,
            radius_km: DEFAULT_RADIUS_KM,
        }
    }
}

impl ModuleConfig {
    /// The config of this install. The platform imports the old KV blob once but skips what its
    /// types reject: a radius stored as a number, a disclaimer stored as a `{fr, en}` map. Until
    /// the host saves the form (the key is then present, even empty), those two are still read
    /// from the KV rather than reset.
    pub fn read(ctx: &Context) -> Result<Self> {
        let mut config = Self::load(ctx)?;
        let Some(held) = ctx.module_config.as_ref().and_then(Value::as_object) else {
            return Ok(config.normalized());
        };
        let missing = |key: &str| held.get(key).is_none_or(Value::is_null);
        if missing("radius_km") || missing("disclaimer") {
            let legacy = portaki_sdk::config::legacy_config()?;
            if missing("radius_km") {
                if let Some(radius) = legacy.get("radius_km").and_then(Value::as_u64) {
                    config.radius_km = u32::try_from(radius).unwrap_or(MAX_RADIUS_KM);
                }
            }
            if missing("disclaimer") {
                if let Some(disclaimer) = legacy.get("disclaimer") {
                    config.disclaimer = Localized::from_value(disclaimer);
                }
            }
        }
        Ok(config.normalized())
    }

    fn normalized(mut self) -> Self {
        self.radius_km = self.normalized_radius_km();
        self
    }

    pub fn is_empty(&self) -> bool {
        self.parse_events().is_empty() && !self.nearby_enabled && self.disclaimer.is_empty()
    }

    /// The filled slots, in form order. A row saved by the host form has no id: it takes its
    /// slot's (`evt-1`…), as the module's own `updateConfig` used to give it.
    pub fn parse_events(&self) -> Vec<EventRow> {
        self.events
            .iter()
            .enumerate()
            .filter(|(_, e)| !e.title.is_empty())
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventRow {
    #[serde(default)]
    pub id: String,
    #[serde(default, deserialize_with = "deserialize_localized_field")]
    pub title: Localized,
    #[serde(default, deserialize_with = "deserialize_localized_field")]
    pub place: Localized,
    #[serde(default)]
    pub starts_at: String,
    #[serde(default)]
    pub ends_at: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nonempty")]
    pub url: Option<String>,
    #[serde(default, deserialize_with = "deserialize_coord")]
    pub lat: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_coord")]
    pub lng: Option<f64>,
    #[serde(default)]
    pub note: Option<Localized>,
}

impl EventRow {
    pub fn has_coords(&self) -> bool {
        match (self.lat, self.lng) {
            (Some(lat), Some(lng)) => lat != 0.0 || lng != 0.0,
            _ => false,
        }
    }
}

/// N-language string map. Legacy `{fr,en}` deserializes as-is; extra langs via flatten.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Localized {
    #[serde(default)]
    pub fr: String,
    #[serde(default)]
    pub en: String,
    #[serde(flatten)]
    pub other: BTreeMap<String, String>,
}

impl Localized {
    pub fn lang_code(locale: &str) -> String {
        let trimmed = locale.trim();
        if trimmed.is_empty() {
            return "fr".to_string();
        }
        let lower = trimmed.to_ascii_lowercase();
        let base = lower.split(['-', '_']).next().unwrap_or("fr").trim();
        if base.is_empty() {
            "fr".to_string()
        } else {
            base.to_string()
        }
    }

    pub fn singleton(lang: &str, value: impl Into<String>) -> Self {
        let mut loc = Self::default();
        loc.set(lang, value.into());
        loc
    }

    pub fn get(&self, lang: &str) -> &str {
        let code = Self::lang_code(lang);
        match code.as_str() {
            "fr" => self.fr.as_str(),
            "en" => self.en.as_str(),
            other => self.other.get(other).map(String::as_str).unwrap_or(""),
        }
    }

    pub fn set(&mut self, lang: &str, value: String) {
        let code = Self::lang_code(lang);
        match code.as_str() {
            "fr" => self.fr = value,
            "en" => self.en = value,
            other => {
                if value.trim().is_empty() {
                    self.other.remove(other);
                } else {
                    self.other.insert(other.to_string(), value);
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.fr.trim().is_empty()
            && self.en.trim().is_empty()
            && self.other.values().all(|v| v.trim().is_empty())
    }

    pub fn pick(&self, locale: &str) -> String {
        self.pick_with_fallback(locale, "fr")
    }

    pub fn pick_with_fallback(&self, guest_locale: &str, property_locale: &str) -> String {
        let candidates = [
            Self::lang_code(guest_locale),
            Self::lang_code(property_locale),
            "fr".to_string(),
        ];
        let mut tried = std::collections::BTreeSet::new();
        for lang in &candidates {
            if !tried.insert(lang.clone()) {
                continue;
            }
            let value = self.get(lang);
            if !value.trim().is_empty() {
                return value.to_string();
            }
        }
        for value in [self.fr.as_str(), self.en.as_str()] {
            if !value.trim().is_empty() {
                return value.to_string();
            }
        }
        for value in self.other.values() {
            if !value.trim().is_empty() {
                return value.clone();
            }
        }
        String::new()
    }

    pub fn from_value(value: &Value) -> Self {
        match value {
            Value::String(s) => Self::singleton("fr", s.trim()),
            Value::Object(map) => {
                let mut loc = Self::default();
                for (key, val) in map {
                    if let Some(s) = val.as_str() {
                        loc.set(key, s.trim().to_string());
                    }
                }
                loc
            }
            _ => Self::default(),
        }
    }
}

fn deserialize_localized_field<'de, D>(deserializer: D) -> std::result::Result<Localized, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    Ok(Localized::from_value(&value))
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

    /// What the host form sends: six slots, every text a string, the select a string.
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
            let config = ModuleConfig::read(&ctx).unwrap();
            let events = config.parse_events();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].id, "evt-2");
            assert_eq!(events[0].title.pick("en"), "Fête du village");
            assert_eq!(events[0].url, None);
            assert_eq!((events[0].lat, events[0].lng), (Some(43.58), Some(7.12)));
            assert_eq!(config.disclaimer.pick("fr"), "d");
            assert!(!config.nearby_enabled);
            assert_eq!(config.radius_km, 20);
        });
    }

    /// The import skips a numeric radius and a `{fr, en}` disclaimer: until the host saves, the
    /// KV still gives them. Once saved (keys present), the platform wins.
    #[test]
    #[serial_test::serial]
    fn a_numeric_radius_and_a_localized_disclaimer_survive_the_import() {
        let legacy = json!({
            "radius_km": 60,
            "disclaimer": { "fr": "Dates indicatives", "en": "Dates are indicative" }
        });
        MockContext::host()
            .with_kv("config", serde_json::to_vec(&legacy).unwrap())
            .with_config(&json!({ "nearby_enabled": true }))
            .run(|ctx| {
                let config = ModuleConfig::read(&ctx).unwrap();
                assert_eq!(config.radius_km, 60);
                assert_eq!(config.disclaimer.get("en"), "Dates are indicative");
            });
        MockContext::host()
            .with_kv("config", serde_json::to_vec(&legacy).unwrap())
            .with_config(&json!({ "radius_km": "10", "disclaimer": "" }))
            .run(|ctx| {
                let config = ModuleConfig::read(&ctx).unwrap();
                assert_eq!(config.radius_km, 10);
                assert!(config.disclaimer.is_empty());
            });
    }
}
