//! Host configuration stored in KV (`config` key).

use std::collections::BTreeMap;

use portaki_sdk::host;
use portaki_sdk::Result;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

const CONFIG_KEY: &str = "config";
const DEFAULT_RADIUS_KM: u32 = 40;
const MIN_RADIUS_KM: u32 = 5;
const MAX_RADIUS_KM: u32 = 100;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleConfig {
    #[serde(default)]
    pub events: Vec<EventRow>,
    #[serde(default, deserialize_with = "deserialize_localized_field")]
    pub disclaimer: Localized,
    /// When true, guest surfaces also load OpenAgenda events around the property.
    #[serde(default = "default_nearby_enabled")]
    pub nearby_enabled: bool,
    /// Search radius in kilometers (bbox approximation for OpenAgenda `geo`).
    #[serde(default = "default_radius_km")]
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
    pub fn is_empty(&self) -> bool {
        self.parse_events().is_empty() && !self.nearby_enabled && self.disclaimer.is_empty()
    }

    pub fn parse_events(&self) -> Vec<EventRow> {
        self.events
            .iter()
            .filter(|e| !e.id.trim().is_empty())
            .cloned()
            .collect()
    }

    pub fn normalized_radius_km(&self) -> u32 {
        self.radius_km.clamp(MIN_RADIUS_KM, MAX_RADIUS_KM)
    }
}

fn default_nearby_enabled() -> bool {
    true
}

fn default_radius_km() -> u32 {
    DEFAULT_RADIUS_KM
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventRow {
    pub id: String,
    pub title: Localized,
    #[serde(default)]
    pub place: Localized,
    #[serde(default)]
    pub starts_at: String,
    #[serde(default)]
    pub ends_at: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub lat: Option<f64>,
    #[serde(default)]
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

pub fn load_config() -> Result<ModuleConfig> {
    let Some(bytes) = host::kv::get(CONFIG_KEY)? else {
        return Ok(ModuleConfig::default());
    };
    let mut config: ModuleConfig = serde_json::from_slice(&bytes).map_err(|error| {
        portaki_sdk::PortakiError::Storage(format!("invalid config JSON: {error}"))
    })?;
    config.radius_km = config.normalized_radius_km();
    Ok(config)
}

pub fn save_config(config: &ModuleConfig) -> Result<()> {
    let mut normalized = config.clone();
    normalized.radius_km = normalized.normalized_radius_km();
    let bytes = serde_json::to_vec(&normalized).map_err(|error| {
        portaki_sdk::PortakiError::Storage(format!("config serialize: {error}"))
    })?;
    host::kv::set(CONFIG_KEY, &bytes, None)
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
