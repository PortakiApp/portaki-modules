//! Host configuration stored in KV (`config` key).

use std::collections::BTreeMap;

use portaki_sdk::host;
use portaki_sdk::Result;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

const CONFIG_KEY: &str = "config";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    #[serde(default)]
    pub spots: Vec<SpotRow>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub spots_json: String,
    /// Per-language disclaimer (plain string legacy → `fr` on load).
    #[serde(default, deserialize_with = "deserialize_localized_field")]
    pub disclaimer: Localized,
}

impl ModuleConfig {
    pub fn is_empty(&self) -> bool {
        self.parse_spots().is_empty() && self.disclaimer.is_empty()
    }

    pub fn parse_spots(&self) -> Vec<SpotRow> {
        self.spots
            .iter()
            .filter(|s| !s.id.trim().is_empty())
            .cloned()
            .collect()
    }

    pub fn migrate_legacy(&mut self) {
        if !self.spots.is_empty() {
            return;
        }
        let raw = self.spots_json.trim();
        if raw.is_empty() {
            return;
        }
        if let Ok(data) = serde_json::from_str::<Vec<SpotRow>>(raw) {
            self.spots = data
                .into_iter()
                .filter(|s| !s.id.trim().is_empty())
                .collect();
            self.spots_json.clear();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpotRow {
    pub id: String,
    pub title: Localized,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub distance: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub note: Option<Localized>,
    #[serde(default)]
    pub detail: Option<Localized>,
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

    /// Guest resolve: request locale → property default → `fr` → first non-empty.
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
        for (lang, value) in [("fr", self.fr.as_str()), ("en", self.en.as_str())] {
            let _ = lang;
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
    config.migrate_legacy();
    Ok(config)
}

pub fn save_config(config: &ModuleConfig) -> Result<()> {
    let bytes = serde_json::to_vec(config).map_err(|error| {
        portaki_sdk::PortakiError::Storage(format!("config serialize: {error}"))
    })?;
    host::kv::set(CONFIG_KEY, &bytes, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn localized_n_lang_roundtrip() {
        let mut loc = Localized::singleton("de", "Hallo");
        loc.set("fr", "Bonjour".into());
        assert_eq!(loc.get("de"), "Hallo");
        assert_eq!(loc.pick_with_fallback("it-IT", "de-DE"), "Hallo");
        let value = serde_json::to_value(&loc).unwrap();
        assert_eq!(value["de"], "Hallo");
        assert_eq!(value["fr"], "Bonjour");
    }

    #[test]
    fn disclaimer_plain_string_migrates() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "disclaimer": "Suggestions non partenaires"
        }))
        .unwrap();
        assert_eq!(config.disclaimer.get("fr"), "Suggestions non partenaires");
    }
}
