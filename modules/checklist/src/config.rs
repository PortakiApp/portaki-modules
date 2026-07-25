//! Host configuration stored in KV (`config` key).
//!
//! Timing controls when the guest checkout checklist becomes available.

use portaki_sdk::host;
use portaki_sdk::Result;
use serde::{Deserialize, Serialize};

const CONFIG_KEY: &str = "config";

/// When the guest checklist becomes available (checkout-relative).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShowWhen {
    /// From booking confirmation (always).
    Always,
    /// From check-in day (during the stay) — default.
    #[default]
    FromCheckin,
    /// 48 h before checkout.
    BeforeCheckout,
    /// On checkout day.
    CheckoutDay,
}

impl ShowWhen {
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "always" | "confirm" => Self::Always,
            "before_checkout" | "before-checkout" | "before" => Self::BeforeCheckout,
            "checkout_day" | "checkout-day" | "checkout" => Self::CheckoutDay,
            _ => Self::FromCheckin,
        }
    }

    pub fn as_wire(&self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::FromCheckin => "from_checkin",
            Self::BeforeCheckout => "before_checkout",
            Self::CheckoutDay => "checkout_day",
        }
    }

    #[allow(dead_code)]
    pub const CHOICE_LIST_WIRE_VALUES: &'static [&'static str] =
        &["always", "from_checkin", "before_checkout", "checkout_day"];
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    #[serde(default)]
    pub show_when: ShowWhen,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            show_when: ShowWhen::FromCheckin,
        }
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
    fn default_is_from_checkin() {
        assert_eq!(ModuleConfig::default().show_when, ShowWhen::FromCheckin);
    }

    #[test]
    fn show_when_choice_list_values_roundtrip() {
        for wire in ShowWhen::CHOICE_LIST_WIRE_VALUES {
            let parsed = ShowWhen::parse(wire);
            assert_eq!(parsed.as_wire(), *wire);
        }
    }
}
