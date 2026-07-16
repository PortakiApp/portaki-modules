//! Host configuration stored in KV (`config` key).

use portaki_sdk::host;
use portaki_sdk::Result;
use serde::{Deserialize, Serialize};

const CONFIG_KEY: &str = "config";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    #[serde(default)]
    pub facilities: Vec<FacilityRow>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub facilities_json: String,
    #[serde(default)]
    pub general_note: String,
}

impl ModuleConfig {
    pub fn is_empty(&self) -> bool {
        self.parse_facilities().is_empty() && self.general_note.trim().is_empty()
    }

    pub fn parse_facilities(&self) -> Vec<FacilityRow> {
        self.facilities
            .iter()
            .filter(|f| !f.id.trim().is_empty())
            .cloned()
            .collect()
    }

    pub fn migrate_legacy(&mut self) {
        if !self.facilities.is_empty() {
            return;
        }
        let raw = self.facilities_json.trim();
        if raw.is_empty() {
            return;
        }
        if let Ok(data) = serde_json::from_str::<Vec<FacilityRow>>(raw) {
            self.facilities = data
                .into_iter()
                .filter(|f| !f.id.trim().is_empty())
                .collect();
            self.facilities_json.clear();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FacilityRow {
    pub id: String,
    pub title: Localized,
    #[serde(default)]
    pub lines: Vec<Localized>,
    #[serde(default)]
    pub hours: Option<String>,
    #[serde(default)]
    pub note: Option<Localized>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Localized {
    #[serde(default)]
    pub fr: String,
    #[serde(default)]
    pub en: String,
}

impl Localized {
    pub fn pick(&self, locale: &str) -> String {
        if locale.to_ascii_lowercase().starts_with("en") {
            if !self.en.trim().is_empty() {
                self.en.clone()
            } else {
                self.fr.clone()
            }
        } else if !self.fr.trim().is_empty() {
            self.fr.clone()
        } else {
            self.en.clone()
        }
    }
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
