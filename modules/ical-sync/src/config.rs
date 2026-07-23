//! Host configuration stored in KV (`config` key).

use portaki_sdk::host;
use portaki_sdk::Result;
use serde::{Deserialize, Serialize};

const CONFIG_KEY: &str = "config";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    #[serde(default)]
    pub ical_url_primary: String,
    #[serde(default)]
    pub ical_url_secondary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_sync_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync_summary: Option<String>,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            ical_url_primary: String::new(),
            ical_url_secondary: String::new(),
            last_sync_at: None,
            sync_summary: None,
        }
    }
}

impl ModuleConfig {
    pub fn primary_url(&self) -> Option<&str> {
        trim_url(&self.ical_url_primary)
    }

    pub fn secondary_url(&self) -> Option<&str> {
        trim_url(&self.ical_url_secondary)
    }

    pub fn has_any_feed(&self) -> bool {
        self.primary_url().is_some() || self.secondary_url().is_some()
    }
}

fn trim_url(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

pub fn load_config() -> Result<ModuleConfig> {
    let Some(bytes) = host::kv::get(CONFIG_KEY)? else {
        return Ok(ModuleConfig::default());
    };
    serde_json::from_slice(&bytes).map_err(|error| {
        portaki_sdk::PortakiError::Storage(format!("invalid config JSON: {error}"))
    })
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

    #[test]
    fn empty_urls_are_ignored() {
        let config = ModuleConfig {
            ical_url_primary: "  ".into(),
            ical_url_secondary: "https://example.com/a.ics".into(),
            ..Default::default()
        };
        assert!(config.primary_url().is_none());
        assert_eq!(config.secondary_url(), Some("https://example.com/a.ics"));
        assert!(config.has_any_feed());
    }
}
