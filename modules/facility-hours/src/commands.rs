//! Module commands — configuration persistence.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{load_config, save_config, FacilityRow, Localized, ModuleConfig};

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FacilityInput {
    #[serde(default)]
    pub name: String,
    /// Legacy dual-locale fields (folded into `name` for the matching lang).
    #[serde(default)]
    pub name_fr: String,
    #[serde(default)]
    pub name_en: String,
    #[serde(default)]
    pub hours: String,
}

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigArgs {
    #[serde(default)]
    pub facilities: Vec<FacilityInput>,
    #[serde(default)]
    pub facilities_json: String,
    #[serde(default)]
    pub general_note: String,
}

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    let lang = Localized::lang_code(&ctx.locale);
    let existing = load_config().unwrap_or_default();
    let facilities = resolve_facilities(&args, &existing.facilities, &lang);
    let mut general_note = existing.general_note;
    general_note.set(&lang, args.general_note.trim().to_string());
    save_config(&ModuleConfig {
        facilities,
        facilities_json: String::new(),
        general_note,
    })
}

fn resolve_facilities(
    args: &UpdateConfigArgs,
    existing: &[FacilityRow],
    lang: &str,
) -> Vec<FacilityRow> {
    if !args.facilities.is_empty() {
        return args
            .facilities
            .iter()
            .enumerate()
            .filter_map(|(index, input)| merge_facility(input, existing.get(index), index, lang))
            .collect();
    }
    let raw = args.facilities_json.trim();
    if raw.is_empty() {
        return Vec::new();
    }
    serde_json::from_str::<Vec<FacilityRow>>(raw)
        .unwrap_or_default()
        .into_iter()
        .filter(|f| !f.id.trim().is_empty())
        .collect()
}

fn merge_facility(
    input: &FacilityInput,
    previous: Option<&FacilityRow>,
    index: usize,
    lang: &str,
) -> Option<FacilityRow> {
    let mut title = previous.map(|p| p.title.clone()).unwrap_or_default();
    let name = input.name.trim();
    if !name.is_empty() {
        title.set(lang, name.to_string());
    } else {
        if !input.name_fr.trim().is_empty() {
            title.set("fr", input.name_fr.trim().to_string());
        }
        if !input.name_en.trim().is_empty() {
            title.set("en", input.name_en.trim().to_string());
        }
    }
    if title.is_empty() {
        return None;
    }
    let hours = input.hours.trim();
    Some(FacilityRow {
        id: previous
            .map(|p| p.id.clone())
            .unwrap_or_else(|| format!("facility-{}", index + 1)),
        title,
        lines: previous.map(|p| p.lines.clone()).unwrap_or_default(),
        hours: if hours.is_empty() {
            None
        } else {
            Some(hours.to_string())
        },
        note: previous.and_then(|p| p.note.clone()),
    })
}
