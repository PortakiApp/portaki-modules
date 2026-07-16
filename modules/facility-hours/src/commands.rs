//! Module commands — configuration persistence.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{save_config, FacilityRow, Localized, ModuleConfig};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FacilityInput {
    #[serde(default)]
    pub name_fr: String,
    #[serde(default)]
    pub name_en: String,
    #[serde(default)]
    pub hours: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigArgs {
    #[serde(default)]
    pub facilities: Vec<FacilityInput>,
    #[serde(default)]
    pub facilities_json: String,
    #[serde(default)]
    pub general_note: String,
}

impl UpdateConfigArgs {
    fn resolve_facilities(&self) -> Vec<FacilityRow> {
        if !self.facilities.is_empty() {
            return self
                .facilities
                .iter()
                .enumerate()
                .filter_map(|(index, input)| facility_from_input(input, index))
                .collect();
        }
        let raw = self.facilities_json.trim();
        if raw.is_empty() {
            return Vec::new();
        }
        serde_json::from_str::<Vec<FacilityRow>>(raw)
            .unwrap_or_default()
            .into_iter()
            .filter(|f| !f.id.trim().is_empty())
            .collect()
    }
}

fn facility_from_input(input: &FacilityInput, index: usize) -> Option<FacilityRow> {
    if input.name_fr.trim().is_empty() && input.name_en.trim().is_empty() {
        return None;
    }
    let hours = input.hours.trim();
    Some(FacilityRow {
        id: format!("facility-{}", index + 1),
        title: Localized {
            fr: input.name_fr.trim().to_string(),
            en: input.name_en.trim().to_string(),
        },
        lines: Vec::new(),
        hours: if hours.is_empty() {
            None
        } else {
            Some(hours.to_string())
        },
        note: None,
    })
}

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(_ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    save_config(&ModuleConfig {
        facilities: args.resolve_facilities(),
        facilities_json: String::new(),
        general_note: args.general_note,
    })
}
