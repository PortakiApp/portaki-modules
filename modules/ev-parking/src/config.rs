//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use portaki_sdk::contracts::i18n::I18nText;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
#[portaki_sdk::config(legacy = legacy)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    #[field(required, label = "host.spotLabel.label")]
    pub spot_label: I18nText,
    #[field(secret, label = "host.chargerPin.label")]
    pub charger_pin: String,
    #[field(secret, label = "host.parkingCode.label")]
    pub parking_code: String,
    #[field(label = "host.mapUrl.label")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map_url: Option<String>,
    #[field(label = "host.instructions.label")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<I18nText>,
    #[field(
        kind = "select",
        options = ["always", "hours_before_24", "day_before_16h", "at_checkin"],
        label = "config.revealPolicy"
    )]
    pub reveal_policy: RevealPolicy,
}

/// The old KV blob spelled two policies the pre-rename way (`hours_before24`, `day_before16h`),
/// which are not options of the select: the platform would not import them.
fn legacy(mut old: Value) -> Value {
    if let Some(policy) = old.get_mut("reveal_policy") {
        if let Ok(parsed) = serde_json::from_value::<RevealPolicy>(policy.clone()) {
            *policy = parsed.as_wire().into();
        }
    }
    old
}

impl ModuleConfig {
    pub fn is_empty(&self) -> bool {
        self.spot_label.is_blank()
            && self.charger_pin.trim().is_empty()
            && self.parking_code.trim().is_empty()
    }

    pub fn map_url_text(&self) -> Option<&str> {
        self.map_url
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }

    /// The spot in `locale`, `None` when blank.
    pub fn spot_text(&self, locale: &str) -> Option<&str> {
        Some(self.spot_label.get(locale).trim()).filter(|s| !s.is_empty())
    }

    /// The instructions in `locale`, `None` when blank.
    pub fn instructions_text(&self, locale: &str) -> Option<&str> {
        self.instructions
            .as_ref()
            .map(|text| text.get(locale).trim())
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

        config.spot_label = I18nText::new("P2 / 14", "");
        assert!(!config.is_empty());

        config = ModuleConfig::default();
        config.charger_pin = "1234".into();
        assert!(!config.is_empty());

        config = ModuleConfig::default();
        config.parking_code = "5678".into();
        assert!(!config.is_empty());

        config = ModuleConfig::default();
        config.map_url = Some("https://maps.example".into());
        config.instructions = Some(I18nText::new("À gauche", "Turn left"));
        assert!(config.is_empty());
    }

    #[test]
    fn legacy_renames_pre_rename_policies_and_keeps_plain_texts() {
        let mapped = legacy(json!({
            "spot_label": "P2",
            "instructions": "À gauche",
            "charger_pin": "4821",
            "reveal_policy": "hours_before24"
        }));
        assert_eq!(
            mapped,
            json!({
                "spot_label": "P2",
                "instructions": "À gauche",
                "charger_pin": "4821",
                "reveal_policy": "hours_before_24"
            })
        );
        assert_eq!(
            legacy(json!({ "reveal_policy": "day_before16h" }))["reveal_policy"],
            "day_before_16h"
        );
        // Already an option, or unknown: left as is.
        assert_eq!(
            legacy(json!({ "reveal_policy": "at_checkin" })),
            json!({ "reveal_policy": "at_checkin" })
        );
        assert_eq!(
            legacy(json!({ "reveal_policy": "weekly" })),
            json!({ "reveal_policy": "weekly" })
        );
        let config: ModuleConfig = serde_json::from_value(mapped).unwrap();
        assert_eq!(config.spot_text("en"), Some("P2"));
        assert_eq!(config.instructions_text("de"), Some("À gauche"));
    }

    #[test]
    #[serial_test::serial]
    fn the_kv_is_read_through_legacy_until_the_platform_holds_the_config() {
        let old = json!({ "spot_label": "P2", "reveal_policy": "hours_before24" });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .run(|ctx| {
                let config = ModuleConfig::load(&ctx).unwrap();
                assert_eq!(config.reveal_policy, RevealPolicy::HoursBefore24);
                assert_eq!(config.spot_text(&ctx.locale), Some("P2"));
            });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .with_config(&json!({}))
            .run(|ctx| {
                assert_eq!(ModuleConfig::load(&ctx).unwrap(), ModuleConfig::default());
            });
    }
}
