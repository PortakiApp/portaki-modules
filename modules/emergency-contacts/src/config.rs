//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use portaki_sdk::contracts::i18n::I18nText;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The keys are the names of the host form fields: the platform takes `updateConfig` itself.
#[portaki_sdk::config(legacy = legacy)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    #[field(label = "config.contacts")]
    pub contacts: Vec<ContactRow>,
    #[field(required, label = "host.phone.label")]
    pub host_visible_phone: String,
}

/// The old KV blob kept the contacts as a JSON string, `contacts_json`, before the form slots.
fn legacy(mut old: Value) -> Value {
    if let Some(old) = old.as_object_mut() {
        let listed = old
            .remove("contacts_json")
            .and_then(|raw| serde_json::from_str::<Value>(raw.as_str()?).ok());
        if let (Some(contacts), false) = (listed, old.contains_key("contacts")) {
            old.insert("contacts".into(), contacts);
        }
    }
    old
}

impl ModuleConfig {
    pub fn is_empty(&self) -> bool {
        self.parse_contacts().is_empty() && self.host_visible_phone.trim().is_empty()
    }

    /// The filled rows, for the guest: the form sends its slots, blank ones included.
    pub fn parse_contacts(&self) -> Vec<ContactRow> {
        self.contacts
            .iter()
            .filter(|c| !c.phone.trim().is_empty() && !c.label.is_blank())
            .cloned()
            .collect()
    }
}

/// A contact. The form sends `label` and `phone` (and `id`); the platform keeps the rest —
/// `note`, `category`, the label's other languages.
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ContactRow {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub label: I18nText,
    pub phone: String,
    pub note: I18nText,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

impl ContactRow {
    /// Nothing the form shows: a slot the host left (or emptied).
    pub fn is_blank(&self) -> bool {
        self.label.is_blank() && self.phone.trim().is_empty()
    }
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
        assert_eq!(contacts[0].label.get("en"), "Pompiers");
    }

    #[test]
    fn legacy_contacts_json_becomes_contacts() {
        let mapped = legacy(json!({
            "contacts_json": r#"[{"id":"samu","label":{"fr":"SAMU","en":"Ambulance"},"phone":"15","note":{"fr":"Gratuit"},"category":"medical"}]"#,
            "host_visible_phone": "+33 6"
        }));
        assert_eq!(
            mapped,
            json!({
                "contacts": [{ "id": "samu", "label": { "fr": "SAMU", "en": "Ambulance" }, "phone": "15",
                               "note": { "fr": "Gratuit" }, "category": "medical" }],
                "host_visible_phone": "+33 6"
            })
        );
        let config: ModuleConfig = serde_json::from_value(mapped).unwrap();
        assert_eq!(config.contacts[0].note.get("en"), "Gratuit");
        assert_eq!(config.contacts[0].category.as_deref(), Some("medical"));
        // Unreadable: nothing to import, nothing invented.
        assert_eq!(legacy(json!({ "contacts_json": "[oops" })), json!({}));
    }

    #[test]
    #[serial_test::serial]
    fn the_kv_is_read_through_legacy_until_the_platform_holds_the_config() {
        let old = json!({
            "contacts_json": r#"[{"id":"samu","label":{"fr":"SAMU","en":"Ambulance"},"phone":"15"}]"#,
            "host_visible_phone": "+33 6"
        });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .run(|ctx| {
                let contacts = ModuleConfig::load(&ctx).unwrap().parse_contacts();
                assert_eq!(contacts[0].label.get("en"), "Ambulance");
            });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .with_config(&json!({}))
            .run(|ctx| assert!(ModuleConfig::load(&ctx).unwrap().contacts.is_empty()));
    }
}
