//! Host configuration stored in KV (`config` key).

use portaki_sdk::host;
use portaki_sdk::Result;
use serde::{Deserialize, Serialize};

const CONFIG_KEY: &str = "config";

/// Owner-configurable module settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    /// Typed bin rows (preferred storage).
    #[serde(default)]
    pub bins: Vec<BinRow>,
    /// Legacy JSON string — migrated into `bins` on load.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub bins_json: String,
    /// Free-text collection schedule.
    #[serde(default)]
    pub collection_schedule: String,
}

impl ModuleConfig {
    /// True when neither bins nor schedule are configured.
    pub fn is_empty(&self) -> bool {
        self.parse_bins().is_empty() && self.collection_schedule.trim().is_empty()
    }

    /// Returns typed bins (after legacy migration).
    pub fn parse_bins(&self) -> Vec<BinRow> {
        self.bins
            .iter()
            .filter(|b| !b.id.trim().is_empty())
            .cloned()
            .collect()
    }

    /// Fills `bins` from legacy `bins_json` when needed.
    pub fn migrate_legacy(&mut self) {
        if !self.bins.is_empty() {
            return;
        }
        let raw = self.bins_json.trim();
        if raw.is_empty() {
            return;
        }
        if let Ok(data) = serde_json::from_str::<Vec<BinRow>>(raw) {
            self.bins = data
                .into_iter()
                .filter(|b| !b.id.trim().is_empty())
                .collect();
            self.bins_json.clear();
        }
    }
}

/// One recycling bin entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BinRow {
    pub id: String,
    pub title: Localized,
    #[serde(default)]
    pub items: Vec<Localized>,
    /// Optional hex color for the bin chip (e.g. `#f4c020`).
    #[serde(default)]
    pub color: Option<String>,
}

/// FR/EN localized string pair.
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

/// Maps host color select values to guest hex colors.
pub fn color_name_to_hex(name: &str) -> Option<String> {
    match name.trim().to_ascii_lowercase().as_str() {
        "yellow" => Some("#f4c020".into()),
        "green" => Some("#3a8a4d".into()),
        "brown" => Some("#8b5a2b".into()),
        "grey" | "gray" => Some("#8b949e".into()),
        other if other.starts_with('#') => Some(other.to_string()),
        _ => None,
    }
}

/// Maps stored hex (or name) back to a select value.
pub fn color_hex_to_name(color: Option<&str>) -> &'static str {
    match color.map(str::trim).unwrap_or("") {
        "#f4c020" | "yellow" => "yellow",
        "#3a8a4d" | "green" => "green",
        "#8b5a2b" | "brown" => "brown",
        "#8b949e" | "grey" | "gray" => "grey",
        _ => "",
    }
}

/// Loads configuration from KV or returns defaults.
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

/// Persists configuration to KV.
pub fn save_config(config: &ModuleConfig) -> Result<()> {
    let bytes = serde_json::to_vec(config).map_err(|error| {
        portaki_sdk::PortakiError::Storage(format!("config serialize: {error}"))
    })?;
    host::kv::set(CONFIG_KEY, &bytes, None)
}
