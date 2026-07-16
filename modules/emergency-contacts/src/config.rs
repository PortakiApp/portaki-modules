//! Host configuration stored in KV (`config` key).

use portaki_sdk::host;
use portaki_sdk::Result;
use serde::{Deserialize, Serialize};

const CONFIG_KEY: &str = "config";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    #[serde(default)]
    pub contacts: Vec<ContactRow>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub contacts_json: String,
    #[serde(default)]
    pub host_visible_phone: String,
}

impl ModuleConfig {
    pub fn is_empty(&self) -> bool {
        self.parse_contacts().is_empty() && self.host_visible_phone.trim().is_empty()
    }

    pub fn parse_contacts(&self) -> Vec<ContactRow> {
        self.contacts
            .iter()
            .filter(|c| !c.id.trim().is_empty() && !c.phone.trim().is_empty())
            .cloned()
            .collect()
    }

    pub fn migrate_legacy(&mut self) {
        if !self.contacts.is_empty() {
            return;
        }
        let raw = self.contacts_json.trim();
        if raw.is_empty() {
            return;
        }
        if let Ok(data) = serde_json::from_str::<Vec<ContactRow>>(raw) {
            self.contacts = data
                .into_iter()
                .filter(|c| !c.id.trim().is_empty() && !c.phone.trim().is_empty())
                .collect();
            self.contacts_json.clear();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContactRow {
    pub id: String,
    pub label: Localized,
    pub phone: String,
    #[serde(default)]
    pub note: Option<Localized>,
    #[serde(default)]
    pub category: Option<String>,
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
