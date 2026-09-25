//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::localized::deserialize_localized_field;

pub use crate::localized::Localized;

/// The keys are the names of the host form fields: the platform takes `updateConfig` itself.
#[portaki_sdk::config]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    #[field(required, label = "config.facilities")]
    pub facilities: Vec<FacilityRow>,
    /// A plain string from the form; a label per language in the old KV.
    #[field(kind = "textarea", label = "host.note.label")]
    #[serde(deserialize_with = "deserialize_localized_field")]
    pub general_note: Localized,
}

impl ModuleConfig {
    /// The config of this install. The import skips what the old KV kept in another shape: the
    /// general note as a label per language, and the `facilities_json` string the form slots
    /// replaced. While the host has not saved that key, it is still read from the KV.
    pub fn read(ctx: &Context) -> Result<Self> {
        let mut config = Self::load(ctx)?;
        let Some(held) = ctx.module_config.as_ref().and_then(|c| c.as_object()) else {
            return Ok(config);
        };
        let (note_held, facilities_held) = (
            held.contains_key("general_note"),
            held.contains_key("facilities"),
        );
        if note_held && facilities_held {
            return Ok(config);
        }
        let legacy = portaki_sdk::config::legacy_config()?;
        if !note_held {
            if let Some(note) = legacy.get("general_note") {
                config.general_note = Localized::from_value(note);
            }
        }
        if !facilities_held {
            if let Some(facilities) = legacy
                .get("facilities_json")
                .and_then(|raw| raw.as_str())
                .and_then(|raw| serde_json::from_str(raw).ok())
            {
                config.facilities = facilities;
            }
        }
        Ok(config)
    }

    pub fn is_empty(&self) -> bool {
        self.parse_facilities().is_empty() && self.general_note.is_empty()
    }

    /// The named rows: the form sends its six slots, blank ones included.
    pub fn parse_facilities(&self) -> Vec<FacilityRow> {
        self.facilities
            .iter()
            .filter(|f| !f.title.is_empty())
            .cloned()
            .collect()
    }
}

/// A row as the host form sends it (`name`, `hours`), or as the KV kept it (`id`, a title per
/// language, `lines`, `note`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FacilityRow {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(
        default,
        alias = "name",
        deserialize_with = "deserialize_localized_field"
    )]
    pub title: Localized,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<Localized>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hours: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<Localized>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::MockContext;
    use serde_json::json;

    #[test]
    fn a_form_row_reads_with_its_name_as_title() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "facilities": [{ "name": "Piscine", "hours": "9 h – 20 h" }, { "name": "", "hours": "" }],
            "general_note": "Horaires indicatifs"
        }))
        .unwrap();
        let facilities = config.parse_facilities();
        assert_eq!(facilities.len(), 1);
        assert_eq!(facilities[0].title.pick("en"), "Piscine");
        assert_eq!(config.general_note.pick("en"), "Horaires indicatifs");
    }

    #[test]
    #[serial_test::serial]
    fn what_the_import_skips_survives() {
        let legacy = json!({
            "facilities_json": r#"[{"id":"pool","title":{"fr":"Piscine","en":"Pool"},"hours":"08:00 – 20:00"}]"#,
            "general_note": { "fr": "Horaires indicatifs", "en": "Indicative hours" }
        });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&legacy).unwrap())
            .with_config(&json!({}))
            .run(|ctx| {
                let config = ModuleConfig::read(&ctx).unwrap();
                assert_eq!(config.parse_facilities()[0].title.pick("en"), "Pool");
                assert_eq!(config.general_note.pick("en"), "Indicative hours");
            });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&legacy).unwrap())
            .with_config(&json!({ "facilities": [], "general_note": "" }))
            .run(|ctx| {
                let config = ModuleConfig::read(&ctx).unwrap();
                assert!(config.facilities.is_empty());
                assert!(config.general_note.is_empty());
            });
    }
}
