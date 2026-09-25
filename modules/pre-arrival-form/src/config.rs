//! Host configuration, held by the platform (`#[portaki_sdk::config]`).
//!
//! Mirrors design `editorPrearrival` / `prearrival-editor-v1`:
//! - when to show the guest form (`show_when`)
//! - which questions are enabled (`ask_*`)

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

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
#[portaki_sdk::config]
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
}

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
        }
    }
}

const QUESTION_KEYS: [&str; 6] = [
    "ask_arrival_time",
    "ask_occasion",
    "ask_allergies",
    "ask_guest_count",
    "ask_special_needs",
    "ask_id_document",
];

impl ModuleConfig {
    /// The config of this install. The KV kept the questions nested under `questions`, which
    /// the platform import skips: until the host saves a question, it is still read from there.
    /// Present, even `false`, the platform's value wins.
    pub fn read(ctx: &Context) -> Result<Self> {
        let config = Self::load(ctx)?;
        let not_held = Map::new();
        let held = match &ctx.module_config {
            Some(Value::Object(held)) => held,
            Some(_) => return Ok(config),
            // `load` read the KV blob, whose questions are nested: same lookup.
            None => &not_held,
        };
        if QUESTION_KEYS.iter().all(|key| held.contains_key(*key)) {
            return Ok(config);
        }
        let Some(Value::Object(old)) = portaki_sdk::config::legacy_config()?
            .get_mut("questions")
            .map(Value::take)
        else {
            return Ok(config);
        };
        let mut merged = serde_json::to_value(&config).map_err(unreadable)?;
        for key in QUESTION_KEYS {
            if let (false, Some(value)) = (held.contains_key(key), old.get(key)) {
                merged[key] = value.clone();
            }
        }
        serde_json::from_value(merged).map_err(unreadable)
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

fn unreadable(error: serde_json::Error) -> PortakiError {
    PortakiError::Storage(format!("config_unreadable: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::MockContext;
    use serde_json::json;

    /// The KV nested the questions under `questions`, which the import skips: they are still
    /// read from there until the host saves them; a saved one wins, even `false`.
    #[test]
    #[serial_test::serial]
    fn nested_questions_survive_the_import() {
        let legacy = json!({
            "show_when": "confirm",
            "questions": { "ask_occasion": false, "ask_id_document": true }
        });
        let read = |held: Option<Value>| {
            let mut mock =
                MockContext::guest().with_kv("config", serde_json::to_vec(&legacy).unwrap());
            if let Some(held) = held {
                mock = mock.with_config(&held);
            }
            mock.run(|ctx| ModuleConfig::read(&ctx).unwrap())
        };

        let before_import = read(None);
        assert_eq!(before_import.show_when, ShowWhen::Confirm);
        assert!(!before_import.ask_occasion);
        assert!(before_import.ask_id_document);
        assert!(before_import.ask_allergies);

        let imported = read(Some(json!({ "show_when": "confirm" })));
        assert_eq!(imported, before_import);

        let saved = read(Some(
            json!({ "ask_occasion": true, "ask_id_document": false }),
        ));
        assert!(saved.ask_occasion);
        assert!(!saved.ask_id_document);
        assert_eq!(saved.show_when, ShowWhen::Before);
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
