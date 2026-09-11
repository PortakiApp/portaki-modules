//! Module commands — configuration persistence.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{load_config, save_config, ContactRow, Localized, ModuleConfig};

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContactInput {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub label_fr: String,
    #[serde(default)]
    pub label_en: String,
    #[serde(default)]
    pub phone: String,
}

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigArgs {
    #[serde(default)]
    pub contacts: Vec<ContactInput>,
    #[serde(default)]
    pub contacts_json: String,
    #[serde(default)]
    pub host_visible_phone: String,
}

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    let lang = Localized::lang_code(&ctx.locale);
    let existing = load_config().unwrap_or_default();
    let contacts = resolve_contacts(&args, &existing.contacts, &lang);
    save_config(&ModuleConfig {
        contacts,
        contacts_json: String::new(),
        host_visible_phone: args.host_visible_phone,
    })
}

fn resolve_contacts(
    args: &UpdateConfigArgs,
    existing: &[ContactRow],
    lang: &str,
) -> Vec<ContactRow> {
    if !args.contacts.is_empty() {
        return args
            .contacts
            .iter()
            .enumerate()
            .filter_map(|(index, input)| merge_contact(input, existing.get(index), index, lang))
            .collect();
    }
    let raw = args.contacts_json.trim();
    if raw.is_empty() {
        return Vec::new();
    }
    serde_json::from_str::<Vec<ContactRow>>(raw)
        .unwrap_or_default()
        .into_iter()
        .filter(|c| !c.id.trim().is_empty() && !c.phone.trim().is_empty())
        .collect()
}

fn merge_contact(
    input: &ContactInput,
    previous: Option<&ContactRow>,
    index: usize,
    lang: &str,
) -> Option<ContactRow> {
    let phone = input.phone.trim();
    if phone.is_empty() {
        return None;
    }
    let mut label = previous.map(|p| p.label.clone()).unwrap_or_default();
    let single = input.label.trim();
    if !single.is_empty() {
        label.set(lang, single.to_string());
    } else {
        if !input.label_fr.trim().is_empty() {
            label.set("fr", input.label_fr.trim().to_string());
        }
        if !input.label_en.trim().is_empty() {
            label.set("en", input.label_en.trim().to_string());
        }
    }
    if label.is_empty() {
        return None;
    }
    Some(ContactRow {
        id: previous
            .map(|p| p.id.clone())
            .unwrap_or_else(|| format!("contact-{}", index + 1)),
        label,
        phone: phone.to_string(),
        note: previous.and_then(|p| p.note.clone()),
        category: previous.and_then(|p| p.category.clone()),
    })
}
