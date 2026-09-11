//! Host configuration stored in KV (`config` key).

use portaki_sdk::host;
use portaki_sdk::prelude::Swatch;
use portaki_sdk::Result;
use serde::{Deserialize, Serialize};

use crate::localized::deserialize_localized_field;

pub use crate::localized::Localized;

const CONFIG_KEY: &str = "config";

/// Owner-configurable module settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    #[serde(default)]
    pub bins: Vec<BinRow>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub bins_json: String,
    #[serde(default, deserialize_with = "deserialize_localized_field")]
    pub collection_schedule: Localized,
}

impl ModuleConfig {
    pub fn is_empty(&self) -> bool {
        self.parse_bins().is_empty() && self.collection_schedule.is_empty()
    }

    pub fn parse_bins(&self) -> Vec<BinRow> {
        self.bins
            .iter()
            .filter(|b| !b.id.trim().is_empty())
            .cloned()
            .collect()
    }

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
    #[serde(default)]
    pub color: Option<String>,
}

/// The bin tints the host can pick — each one a [`Swatch`] the booklet resolves in its theme.
///
/// A name, never a hex: the yellow bin is yellow because the municipality says so, but which
/// yellow is the shell's call — its palette, a dark theme, contrast.
const BIN_SWATCHES: [(&str, Swatch); 4] = [
    ("yellow", Swatch::Yellow),
    ("green", Swatch::Green),
    ("brown", Swatch::Brown),
    ("grey", Swatch::Grey),
];

/// The hex values stored before swatches, read back as the tint they stood for.
const LEGACY_HEX: [(&str, &str); 4] = [
    ("#f4c020", "yellow"),
    ("#3a8a4d", "green"),
    ("#8b5a2b", "brown"),
    ("#8b949e", "grey"),
];

/// The canonical tint name for a stored or submitted value — `None` for anything else.
///
/// Accepts the names the host select sends, `gray`, and the hex strings earlier versions
/// stored, so configurations saved before swatches keep their dots.
pub fn bin_color_name(value: Option<&str>) -> Option<&'static str> {
    let value = value?.trim().to_ascii_lowercase();
    let value = if value == "gray" {
        "grey".to_string()
    } else {
        value
    };
    LEGACY_HEX
        .iter()
        .find(|(hex, _)| *hex == value)
        .map(|(_, name)| *name)
        .or_else(|| {
            BIN_SWATCHES
                .iter()
                .find(|(name, _)| *name == value)
                .map(|(name, _)| *name)
        })
}

/// The swatch a bin's dot takes, if its color is one the booklet knows.
pub fn bin_swatch(value: Option<&str>) -> Option<Swatch> {
    let name = bin_color_name(value)?;
    BIN_SWATCHES
        .iter()
        .find(|(candidate, _)| *candidate == name)
        .map(|(_, swatch)| *swatch)
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
mod bin_color_tests {
    use super::*;

    #[test]
    fn a_host_name_is_its_own_swatch() {
        assert_eq!(bin_color_name(Some("yellow")), Some("yellow"));
        assert_eq!(bin_color_name(Some(" Gray ")), Some("grey"));
        assert_eq!(bin_swatch(Some("brown")), Some(Swatch::Brown));
    }

    /// Configurations saved before swatches stored hex: their bins keep their dots.
    #[test]
    fn a_legacy_hex_reads_as_the_tint_it_stood_for() {
        assert_eq!(bin_color_name(Some("#F4C020")), Some("yellow"));
        assert_eq!(bin_swatch(Some("#3a8a4d")), Some(Swatch::Green));
    }

    /// An arbitrary color is not a tint the booklet knows — no dot rather than a guessed one.
    #[test]
    fn anything_else_has_no_swatch() {
        assert_eq!(bin_swatch(Some("#123456")), None);
        assert_eq!(bin_swatch(Some("")), None);
        assert_eq!(bin_swatch(None), None);
    }
}
