//! Host configuration, held by the platform (`#[portaki_sdk::config]`).
//!
//! Mirrors design `editorPrearrival` / `prearrival-editor-v1`:
//! - when to show the guest form (`show_when`)
//! - which questions are enabled (`ask_*`)

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// When the guest form becomes available.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShowWhen {
    /// From booking confirmation (as soon as the stay exists).
    Confirm,
    /// 48 h before check-in.
    #[default]
    Before,
    /// On check-in day.
    Checkin,
}

impl ShowWhen {
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "confirm" => Self::Confirm,
            "checkin" | "at_checkin" | "at-checkin" => Self::Checkin,
            _ => Self::Before,
        }
    }

    pub fn as_wire(&self) -> &'static str {
        match self {
            Self::Confirm => "confirm",
            Self::Before => "before",
            Self::Checkin => "checkin",
        }
    }

    pub const CHOICE_LIST_WIRE_VALUES: &'static [&'static str] = &["confirm", "before", "checkin"];
}

/// The keys are the names of the host form fields: the platform takes `updateConfig` itself.
/// The questions sit flat, as the form sends them; the KV kept them under `questions`.
#[portaki_sdk::config(legacy = legacy)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    #[field(
        kind = "select",
        options = ["confirm", "before", "checkin"],
        label = "host.when"
    )]
    pub show_when: ShowWhen,
    #[field(label = "host.question.arrival")]
    pub ask_arrival_time: bool,
    #[field(label = "host.question.occasion")]
    pub ask_occasion: bool,
    #[field(label = "host.question.allergies")]
    pub ask_allergies: bool,
    #[field(label = "host.question.guestCount")]
    pub ask_guest_count: bool,
    #[field(label = "host.question.specialNeeds")]
    pub ask_special_needs: bool,
    #[field(label = "host.question.idDocument")]
    pub ask_id_document: bool,
    /// « Vous arrivez en ? » — voiture, train, avion (§2.20).
    ///
    /// Elle décide ce que l'hôte prépare : une place de parking, un horaire de train, un
    /// transfert d'aéroport. Éteinte par défaut, comme toute question neuve : un hôte qui n'a
    /// pas demandé une question ne doit pas la voir apparaître dans son formulaire.
    #[field(label = "config.askTransport")]
    pub ask_transport: bool,
    /// Le pas des créneaux d'arrivée : `ranges` (les trois plages d'avant ce réglage), ou `15`,
    /// `30`, `60` minutes de l'heure d'entrée à [`Self::slots_until`] (spec Pré-arrivée §2.1).
    #[field(
        kind = "select",
        options = ["ranges", "15", "30", "60"],
        label = "host.slots.step"
    )]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub slot_step: String,
    /// Le dernier créneau, `HH:MM` ; vide : 23:00.
    #[field(label = "host.slots.until")]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub slots_until: String,
}

/// Le dernier créneau proposé quand l'hôte n'en donne pas.
pub const DEFAULT_SLOTS_UNTIL: &str = "23:00";

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            show_when: ShowWhen::Before,
            ask_arrival_time: true,
            ask_occasion: true,
            ask_allergies: true,
            ask_guest_count: true,
            ask_special_needs: false,
            ask_id_document: false,
            ask_transport: false,
            slot_step: String::new(),
            slots_until: String::new(),
        }
    }
}

/// The old KV blob nested the questions under `questions`: lift them to the declared keys
/// (a flat key, already there, wins).
fn legacy(mut old: Value) -> Value {
    if let Some(blob) = old.as_object_mut() {
        if let Some(Value::Object(questions)) = blob.remove("questions") {
            for (key, value) in questions {
                blob.entry(key).or_insert(value);
            }
        }
    }
    old
}

impl ModuleConfig {
    /// Les créneaux d'arrivée d'un logement qui ouvre à `hour` : les trois plages, ou des
    /// créneaux réguliers jusqu'à l'heure de fin.
    pub fn slots(&self, hour: u32) -> Vec<crate::slots::Slot> {
        let step = self.slot_step.trim().parse::<u32>().ok();
        let until = crate::slots::minutes_of(self.slots_until.trim())
            .or_else(|| crate::slots::minutes_of(DEFAULT_SLOTS_UNTIL))
            .unwrap_or(23 * 60);
        match step {
            Some(step) => crate::slots::stepped(hour, until, step),
            None => crate::slots::slots(hour),
        }
    }

    /// Ce qui ne va pas, champ par champ — sous le champ, et dans `publishReadiness`.
    pub fn problems(&self) -> Vec<(&'static str, portaki_sdk::contracts::i18n::I18nText)> {
        portaki_sdk::config::check::time(self.slots_until.trim())
            .map(|error| ("slots_until", error))
            .into_iter()
            .collect()
    }

    /// At least one question is asked.
    pub fn asks_anything(&self) -> bool {
        self.ask_arrival_time
            || self.ask_occasion
            || self.ask_allergies
            || self.ask_guest_count
            || self.ask_special_needs
            || self.ask_id_document
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::MockContext;
    use serde_json::json;

    #[test]
    fn legacy_nested_questions_are_lifted() {
        let mapped = legacy(json!({
            "show_when": "confirm",
            "questions": { "ask_occasion": false, "ask_id_document": true }
        }));
        assert_eq!(
            mapped,
            json!({ "show_when": "confirm", "ask_occasion": false, "ask_id_document": true })
        );
        // A flat key already there wins; a non-object `questions` brings nothing.
        assert_eq!(
            legacy(json!({ "ask_occasion": true, "questions": { "ask_occasion": false } })),
            json!({ "ask_occasion": true })
        );
        assert_eq!(legacy(json!({ "questions": "oops" })), json!({}));
    }

    /// The KV is read (through `legacy`) only while the platform sends no config; `{}` is a
    /// real, empty config.
    #[test]
    #[serial_test::serial]
    fn the_kv_is_read_through_legacy_until_the_platform_holds_the_config() {
        let old = json!({
            "show_when": "confirm",
            "questions": { "ask_occasion": false, "ask_id_document": true }
        });
        let kv = serde_json::to_vec(&old).unwrap();
        let before_import = MockContext::guest()
            .with_kv("config", kv.clone())
            .run(|ctx| ModuleConfig::load(&ctx).unwrap());
        assert_eq!(before_import.show_when, ShowWhen::Confirm);
        assert!(!before_import.ask_occasion);
        assert!(before_import.ask_id_document);
        assert!(before_import.ask_allergies);

        let held = MockContext::guest()
            .with_kv("config", kv)
            .with_config(&json!({}))
            .run(|ctx| ModuleConfig::load(&ctx).unwrap());
        assert_eq!(held, ModuleConfig::default());
    }

    #[test]
    fn default_matches_design_sample() {
        let cfg = ModuleConfig::default();
        assert_eq!(cfg.show_when, ShowWhen::Before);
        assert!(cfg.ask_arrival_time);
        assert!(cfg.ask_occasion);
        assert!(cfg.ask_allergies);
        assert!(cfg.ask_guest_count);
        assert!(!cfg.ask_special_needs);
        assert!(!cfg.ask_id_document);
    }

    #[test]
    fn show_when_choice_list_values_deserialize() {
        for wire in ShowWhen::CHOICE_LIST_WIRE_VALUES {
            let parsed = ShowWhen::parse(wire);
            assert_eq!(parsed.as_wire(), *wire);
        }
    }
}
