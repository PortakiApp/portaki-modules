//! Host configuration stored in KV (`config` key).

use portaki_sdk::host;
use portaki_sdk::Result;
use serde::{Deserialize, Serialize};

const CONFIG_KEY: &str = "config";

#[portaki_sdk::params]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RevealPolicy {
    Always,
    #[serde(rename = "hours_before_24", alias = "hours_before24")]
    HoursBefore24,
    #[default]
    #[serde(rename = "day_before_16h", alias = "day_before16h")]
    DayBefore16h,
    AtCheckin,
}

impl RevealPolicy {
    pub const fn as_wire(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::HoursBefore24 => "hours_before_24",
            Self::DayBefore16h => "day_before_16h",
            Self::AtCheckin => "at_checkin",
        }
    }

    pub const CHOICE_LIST_WIRE_VALUES: &[&str] =
        &["always", "hours_before_24", "day_before_16h", "at_checkin"];

    pub const ALL: &[RevealPolicy] = &[
        Self::Always,
        Self::HoursBefore24,
        Self::DayBefore16h,
        Self::AtCheckin,
    ];
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    #[serde(default)]
    pub spot_label: String,
    #[serde(default)]
    pub charger_pin: String,
    #[serde(default)]
    pub parking_code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub map_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(default)]
    pub reveal_policy: RevealPolicy,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            spot_label: String::new(),
            charger_pin: String::new(),
            parking_code: String::new(),
            map_url: None,
            instructions: None,
            reveal_policy: RevealPolicy::DayBefore16h,
        }
    }
}

impl ModuleConfig {
    pub fn is_empty(&self) -> bool {
        self.spot_label.trim().is_empty()
            && self.charger_pin.trim().is_empty()
            && self.parking_code.trim().is_empty()
    }

    pub fn map_url_text(&self) -> Option<&str> {
        self.map_url
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }

    pub fn instructions_text(&self) -> Option<&str> {
        self.instructions
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
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
    use serde_json::json;

    #[test]
    fn default_reveal_policy_is_day_before_16h() {
        assert_eq!(
            ModuleConfig::default().reveal_policy,
            RevealPolicy::DayBefore16h
        );
    }

    #[test]
    fn reveal_policy_choice_list_values_deserialize() {
        for wire in RevealPolicy::CHOICE_LIST_WIRE_VALUES {
            let parsed: RevealPolicy = serde_json::from_value(json!(wire)).unwrap_or_else(|e| {
                panic!("ChoiceList reveal_policy value {wire:?} must deserialize: {e}")
            });
            assert_eq!(parsed.as_wire(), *wire);
        }
    }

    #[test]
    fn is_empty_requires_spot_pin_and_code() {
        let mut config = ModuleConfig::default();
        assert!(config.is_empty());

        config.spot_label = "P2 / 14".into();
        assert!(!config.is_empty());

        config = ModuleConfig::default();
        config.charger_pin = "1234".into();
        assert!(!config.is_empty());

        config = ModuleConfig::default();
        config.parking_code = "5678".into();
        assert!(!config.is_empty());

        config = ModuleConfig::default();
        config.map_url = Some("https://maps.example".into());
        config.instructions = Some("Turn left".into());
        assert!(config.is_empty());
    }
}
