//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use portaki_sdk::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

pub use crate::localized::Localized;

/// The keys are the names of the host form fields: the platform takes `updateConfig` itself.
#[portaki_sdk::config]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    #[field(label = "config.contacts")]
    pub contacts: Vec<ContactRow>,
    #[field(required, label = "host.phone.label")]
    pub host_visible_phone: String,
}

impl ModuleConfig {
    /// The config of this install. The platform imports only `contacts`, not the older
    /// `contacts_json` string the form slots replaced: while the host has not saved `contacts`,
    /// that list is still read from the KV rather than lost.
    pub fn read(ctx: &Context) -> Result<Self> {
        let mut config = Self::load(ctx)?;
        let held = ctx.module_config.as_ref().and_then(|c| c.as_object());
        if held.is_some_and(|held| !held.contains_key("contacts")) {
            if let Some(contacts) = portaki_sdk::config::legacy_config()?
                .get("contacts_json")
                .and_then(Value::as_str)
                .and_then(|raw| serde_json::from_str(raw).ok())
            {
                config.contacts = contacts;
            }
        }
        Ok(config)
    }

    pub fn is_empty(&self) -> bool {
        self.parse_contacts().is_empty() && self.host_visible_phone.trim().is_empty()
    }

    /// The filled rows: the form sends its six slots, blank ones included.
    pub fn parse_contacts(&self) -> Vec<ContactRow> {
        self.contacts
            .iter()
            .filter(|c| !c.phone.trim().is_empty() && !c.label.is_empty())
            .cloned()
            .collect()
    }
}

/// A row as the host form sends it (`label`, `phone`), or as the KV kept it (`id`, a label per
/// language, `note`, `category`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContactRow {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(default, deserialize_with = "localized")]
    pub label: Localized,
    #[serde(default)]
    pub phone: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<Localized>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// A plain string (the form) or a per-language object (the KV).
fn localized<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Localized, D::Error> {
    Ok(Localized::from_value(&Value::deserialize(deserializer)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::MockContext;
    use serde_json::json;

    #[test]
    fn a_form_row_reads_with_a_plain_label() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "contacts": [{ "label": "Pompiers", "phone": "18" }, { "label": "", "phone": "" }],
            "host_visible_phone": "+33 6"
        }))
        .unwrap();
        let contacts = config.parse_contacts();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].label.pick("en"), "Pompiers");
    }

    #[test]
    #[serial_test::serial]
    fn contacts_json_survives_the_import() {
        let legacy = json!({
            "contacts_json": r#"[{"id":"samu","label":{"fr":"SAMU","en":"Ambulance"},"phone":"15"}]"#,
            "host_visible_phone": "+33 6"
        });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&legacy).unwrap())
            .with_config(&json!({ "host_visible_phone": "+33 6" }))
            .run(|ctx| {
                let contacts = ModuleConfig::read(&ctx).unwrap().parse_contacts();
                assert_eq!(contacts.len(), 1);
                assert_eq!(contacts[0].label.pick("en"), "Ambulance");
            });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&legacy).unwrap())
            .with_config(&json!({ "contacts": [], "host_visible_phone": "+33 6" }))
            .run(|ctx| assert!(ModuleConfig::read(&ctx).unwrap().contacts.is_empty()));
    }
}
