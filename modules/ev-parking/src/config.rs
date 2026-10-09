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
// Pas d'`Eq` : la puissance est un flottant.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ModuleConfig {
    #[field(required, label = "host.spotLabel.label")]
    pub spot_label: I18nText,
    #[field(
        secret,
        reveal(guest_pre_arrival, guest_stay),
        label = "host.chargerPin.label"
    )]
    pub charger_pin: String,
    #[field(
        secret,
        reveal(guest_pre_arrival, guest_stay),
        label = "host.parkingCode.label"
    )]
    pub parking_code: String,
    #[field(kind = "url", label = "host.mapUrl.label")]
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
    // §2.2 La borne.
    /// `type2`, `ccs`, `domestic` ou `other` ([`CHARGER_TYPES`]) ; vide : Type 2.
    #[field(
        kind = "select",
        options = ["type2", "ccs", "domestic", "other"],
        label = "host.chargerType.label"
    )]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub charger_type: String,
    /// La puissance en kW ; `0` : non renseignée. Flottant : le `NumberInput` envoie un nombre.
    #[field(label = "host.power.label")]
    #[serde(default, skip_serializing_if = "is_zero")]
    pub power_kw: f64,
    /// Câble fourni. Absent : oui.
    #[field(label = "host.cable.label")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cable_provided: Option<bool>,
    // §2.3 Tarif et réservation.
    /// `included`, `per_kwh`, `per_stay` ou `on_site` ([`PRICINGS`]) ; vide : incluse.
    #[field(
        kind = "select",
        options = ["included", "per_kwh", "per_stay", "on_site"],
        label = "host.pricing.label"
    )]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub pricing: String,
    /// Le prix, tel qu'écrit : « 0,25 €/kWh ». Obligatoire quand la recharge n'est pas incluse.
    #[field(label = "host.price.label")]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub price: String,
    /// La borne se réserve (partagée avec d'autres logements).
    #[field(label = "host.booking.label")]
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub booking_required: bool,
    /// Comment réserver : « Prévenez-moi la veille ».
    #[field(label = "host.bookingNote.label")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub booking_note: Option<I18nText>,
}

fn is_zero(value: &f64) -> bool {
    *value == 0.0
}

/// Les prises (§2.2), Type 2 d'abord : c'est le défaut.
pub const CHARGER_TYPES: [&str; 4] = ["type2", "ccs", "domestic", "other"];
/// Les tarifs (§2.3), incluse d'abord : c'est le défaut.
pub const PRICINGS: [&str; 4] = ["included", "per_kwh", "per_stay", "on_site"];
/// Les bornes de la puissance (§2.2).
pub const POWER_KW: (f64, f64) = (2.3, 350.0);

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
    pub fn charger_type(&self) -> &'static str {
        CHARGER_TYPES
            .iter()
            .find(|key| **key == self.charger_type.trim())
            .unwrap_or(&CHARGER_TYPES[0])
    }

    pub fn pricing(&self) -> &'static str {
        PRICINGS
            .iter()
            .find(|key| **key == self.pricing.trim())
            .unwrap_or(&PRICINGS[0])
    }

    pub fn cable_provided(&self) -> bool {
        self.cable_provided.unwrap_or(true)
    }

    /// La puissance, quand elle est renseignée et plausible.
    pub fn power(&self) -> Option<f64> {
        (self.power_kw.is_finite() && self.power_kw > 0.0).then_some(self.power_kw)
    }

    /// Ce qui ne va pas, champ par champ — sous le champ, et dans `publishReadiness`.
    pub fn problems(&self) -> Vec<(&'static str, I18nText)> {
        use portaki_sdk::config::check;
        let text = crate::i18n::text;
        let too_long = |value: Option<&I18nText>, max: usize| {
            value.and_then(|value| {
                value
                    .by_language()
                    .find_map(|(_, text)| check::max_chars(text, max))
            })
        };
        let mut problems: Vec<(&'static str, I18nText)> = Vec::new();
        let mut push = |field: &'static str, error: Option<I18nText>| {
            if let Some(error) = error {
                problems.push((field, error));
            }
        };
        // Vide, l'emplacement est refusé par la plateforme (`required`) : ici, la longueur.
        push("spot_label", too_long(Some(&self.spot_label), 60));
        push(
            "power_kw",
            (self.power_kw != 0.0)
                .then(|| check::between(self.power_kw, POWER_KW.0, POWER_KW.1))
                .flatten()
                .map(|_| text("host.power.range")),
        );
        push(
            "price",
            if self.pricing() != "included" && self.price.trim().is_empty() {
                Some(text("host.price.required"))
            } else {
                check::max_chars(&self.price, 30)
            },
        );
        push("instructions", too_long(self.instructions.as_ref(), 280));
        if self.booking_required {
            push("booking_note", too_long(self.booking_note.as_ref(), 120));
        }
        problems
    }

    /// Le message à afficher sous `field`, s'il y en a un.
    pub fn error_of(&self, field: &str) -> Option<I18nText> {
        self.problems()
            .into_iter()
            .find(|(name, _)| *name == field)
            .map(|(_, error)| error)
    }

    pub fn is_empty(&self) -> bool {
        self.spot_label.is_blank()
            && self.charger_pin.trim().is_empty()
            && self.parking_code.trim().is_empty()
    }

    /// The map link, `https` only: a guest never gets a link in clear, nor a `javascript:` one.
    pub fn map_url_text(&self) -> Option<&str> {
        self.map_url
            .as_deref()
            .map(str::trim)
            .filter(|s| s.starts_with("https://"))
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
    fn only_an_https_map_url_is_a_link() {
        let with = |url: &str| ModuleConfig {
            map_url: Some(url.into()),
            ..ModuleConfig::default()
        };
        assert_eq!(
            with(" https://maps.example/p ").map_url_text(),
            Some("https://maps.example/p")
        );
        for refused in [
            "",
            "Texte de test",
            "http://maps.example",
            "javascript:alert(1)",
        ] {
            assert_eq!(with(refused).map_url_text(), None, "{refused}");
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

    /// Le prix n'est exigé que si la recharge n'est pas incluse ; la puissance est bornée.
    #[test]
    fn problems_follow_the_pricing_and_the_power() {
        let config = |value| serde_json::from_value::<ModuleConfig>(value).unwrap();
        let fields = |config: ModuleConfig| -> Vec<&'static str> {
            config.problems().into_iter().map(|(f, _)| f).collect()
        };
        assert!(fields(config(json!({ "spot_label": "P2" }))).is_empty());
        assert_eq!(
            fields(config(
                json!({ "spot_label": "P2", "pricing": "per_kwh", "power_kw": 1.0 })
            )),
            ["power_kw", "price"]
        );
        assert_eq!(config(json!({})).charger_type(), "type2");
        assert!(config(json!({})).cable_provided());
    }
}
