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
/// No password is only recommended: an open network (captive portal) has none.
#[portaki_sdk::config(legacy = legacy)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    #[field(required, label = "host.ssid.label")]
    pub ssid: String,
    #[field(secret, recommended, label = "host.password.label")]
    pub password: String,
    #[field(label = "host.hint.label")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<I18nText>,
    #[field(label = "host.connectionSteps.label")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_steps: Option<I18nText>,
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
        self.ssid.trim().is_empty() && self.password.trim().is_empty()
    }

    /// The hint in `locale`, `None` when blank.
    pub fn hint_text(&self, locale: &str) -> Option<&str> {
        text_in(self.hint.as_ref(), locale)
    }

    /// The connection steps in `locale`, `None` when blank.
    pub fn connection_steps_text(&self, locale: &str) -> Option<&str> {
        text_in(self.connection_steps.as_ref(), locale)
    }
}

fn text_in<'a>(text: Option<&'a I18nText>, locale: &str) -> Option<&'a str> {
    text.map(|text| text.get(locale).trim())
        .filter(|s| !s.is_empty())
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
    fn legacy_renames_pre_rename_policies_and_keeps_plain_texts() {
        let mapped = legacy(json!({
            "ssid": "Villa",
            "hint": "5 GHz au salon",
            "connection_steps": "Choisir Villa",
            "reveal_policy": "hours_before24"
        }));
        assert_eq!(
            mapped,
            json!({
                "ssid": "Villa",
                "hint": "5 GHz au salon",
                "connection_steps": "Choisir Villa",
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
        assert_eq!(config.hint_text("en"), Some("5 GHz au salon"));
        assert_eq!(config.connection_steps_text("de"), Some("Choisir Villa"));
    }

    #[test]
    #[serial_test::serial]
    fn the_kv_is_read_through_legacy_until_the_platform_holds_the_config() {
        let old = json!({ "ssid": "Villa", "reveal_policy": "hours_before24" });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .run(|ctx| {
                let config = ModuleConfig::load(&ctx).unwrap();
                assert_eq!(config.reveal_policy, RevealPolicy::HoursBefore24);
                assert_eq!(config.ssid, "Villa");
            });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .with_config(&json!({}))
            .run(|ctx| {
                assert_eq!(ModuleConfig::load(&ctx).unwrap(), ModuleConfig::default());
            });
    }
}
