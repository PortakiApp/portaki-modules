//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

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

/// The keys are the names of the host form fields: the platform takes `updateConfig` itself.
#[portaki_sdk::config]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    #[field(required, label = "host.spotLabel.label")]
    pub spot_label: String,
    #[field(secret, label = "host.chargerPin.label")]
    pub charger_pin: String,
    #[field(secret, label = "host.parkingCode.label")]
    pub parking_code: String,
    #[field(label = "host.mapUrl.label")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map_url: Option<String>,
    #[field(kind = "textarea", label = "host.instructions.label")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[field(
        kind = "select",
        options = ["always", "hours_before_24", "day_before_16h", "at_checkin"],
        label = "config.revealPolicy"
    )]
    pub reveal_policy: RevealPolicy,
}

impl ModuleConfig {
    /// The config of this install. The platform imports the old KV blob once, but skips a policy
    /// spelled the pre-rename way (`hours_before24`, `day_before16h`), which is not one of the
    /// options: until the host saves the form, that policy is still read from the KV.
    pub fn read(ctx: &Context) -> Result<Self> {
        let mut config = Self::load(ctx)?;
        let held = ctx.module_config.as_ref().and_then(|c| c.as_object());
        if held.is_some_and(|held| !held.contains_key("reveal_policy")) {
            if let Some(policy) = portaki_sdk::config::legacy_config()?
                .get("reveal_policy")
                .and_then(|raw| serde_json::from_value(raw.clone()).ok())
            {
                config.reveal_policy = policy;
            }
        }
        Ok(config)
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::MockContext;
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

    #[test]
    #[serial_test::serial]
    fn a_pre_rename_policy_survives_the_import() {
        let legacy = json!({ "spot_label": "P2", "reveal_policy": "hours_before24" });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&legacy).unwrap())
            .with_config(&json!({ "spot_label": "P2" }))
            .run(|ctx| {
                let config = ModuleConfig::read(&ctx).unwrap();
                assert_eq!(config.reveal_policy, RevealPolicy::HoursBefore24);
            });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&legacy).unwrap())
            .with_config(&json!({ "spot_label": "P2", "reveal_policy": "always" }))
            .run(|ctx| {
                assert_eq!(
                    ModuleConfig::read(&ctx).unwrap().reveal_policy,
                    RevealPolicy::Always
                );
            });
    }
}
