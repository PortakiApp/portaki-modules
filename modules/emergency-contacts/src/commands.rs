//! Module commands — configuration persistence.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{save_config, ContactRow, Localized, ModuleConfig};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContactInput {
    #[serde(default)]
    pub label_fr: String,
    #[serde(default)]
    pub label_en: String,
    #[serde(default)]
    pub phone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigArgs {
    #[serde(default)]
    pub contacts: Vec<ContactInput>,
    #[serde(default)]
    pub contacts_json: String,
    #[serde(default)]
    pub host_visible_phone: String,
}

impl UpdateConfigArgs {
    fn resolve_contacts(&self) -> Vec<ContactRow> {
        if !self.contacts.is_empty() {
            return self
                .contacts
                .iter()
                .enumerate()
                .filter_map(|(index, input)| contact_from_input(input, index))
                .collect();
        }
        let raw = self.contacts_json.trim();
        if raw.is_empty() {
            return Vec::new();
        }
        serde_json::from_str::<Vec<ContactRow>>(raw)
            .unwrap_or_default()
            .into_iter()
            .filter(|c| !c.id.trim().is_empty() && !c.phone.trim().is_empty())
            .collect()
    }
}

fn contact_from_input(input: &ContactInput, index: usize) -> Option<ContactRow> {
    let label_empty = input.label_fr.trim().is_empty() && input.label_en.trim().is_empty();
    let phone = input.phone.trim();
    if label_empty || phone.is_empty() {
        return None;
    }
    Some(ContactRow {
        id: format!("contact-{}", index + 1),
        label: Localized {
            fr: input.label_fr.trim().to_string(),
            en: input.label_en.trim().to_string(),
        },
        phone: phone.to_string(),
        note: None,
        category: None,
    })
}

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(_ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    save_config(&ModuleConfig {
        contacts: args.resolve_contacts(),
        contacts_json: String::new(),
        host_visible_phone: args.host_visible_phone,
    })
}
