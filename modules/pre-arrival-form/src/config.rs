//! Host configuration stored in KV (`config` key).
//!
//! Mirrors design `editorPrearrival` / `prearrival-editor-v1`:
//! - when to show the guest form (`show_when`)
//! - which questions are enabled (`ask_*`)

use portaki_sdk::host;
use portaki_sdk::Result;
use serde::{Deserialize, Serialize};

const CONFIG_KEY: &str = "config";

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

    pub const CHOICE_LIST_WIRE_VALUES: &'static [&'static str] =
        &["confirm", "before", "checkin"];
}

/// Which guest form questions are active.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FormQuestions {
    #[serde(default = "default_true")]
    pub ask_arrival_time: bool,
    #[serde(default = "default_true")]
    pub ask_occasion: bool,
    #[serde(default = "default_true")]
    pub ask_allergies: bool,
    #[serde(default = "default_true")]
    pub ask_guest_count: bool,
    #[serde(default)]
    pub ask_special_needs: bool,
    #[serde(default)]
    pub ask_id_document: bool,
}

impl Default for FormQuestions {
    fn default() -> Self {
        Self {
            ask_arrival_time: true,
            ask_occasion: true,
            ask_allergies: true,
            ask_guest_count: true,
            ask_special_needs: false,
            ask_id_document: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    #[serde(default)]
    pub show_when: ShowWhen,
    #[serde(default)]
    pub questions: FormQuestions,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            show_when: ShowWhen::Before,
            questions: FormQuestions::default(),
        }
    }
}

fn default_true() -> bool {
    true
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
    fn default_matches_design_sample() {
        let cfg = ModuleConfig::default();
        assert_eq!(cfg.show_when, ShowWhen::Before);
        assert!(cfg.questions.ask_arrival_time);
        assert!(cfg.questions.ask_occasion);
        assert!(cfg.questions.ask_allergies);
        assert!(cfg.questions.ask_guest_count);
        assert!(!cfg.questions.ask_special_needs);
        assert!(!cfg.questions.ask_id_document);
    }

    #[test]
    fn show_when_choice_list_values_deserialize() {
        for wire in ShowWhen::CHOICE_LIST_WIRE_VALUES {
            let parsed = ShowWhen::parse(wire);
            assert_eq!(parsed.as_wire(), *wire);
        }
    }
}
