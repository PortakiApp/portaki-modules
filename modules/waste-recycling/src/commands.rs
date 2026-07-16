//! Module commands — configuration persistence.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{color_name_to_hex, save_config, BinRow, Localized, ModuleConfig};

/// One bin row from the host form (`bins.N.*`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BinInput {
    #[serde(default)]
    pub title_fr: String,
    #[serde(default)]
    pub title_en: String,
    #[serde(default)]
    pub items_fr: String,
    #[serde(default)]
    pub color: String,
}

/// Arguments for `updateConfig`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigArgs {
    /// Nested array from dashboard form merge.
    #[serde(default)]
    pub bins: Vec<BinInput>,
    /// Legacy JSON fallback (unused by structured host UI).
    #[serde(default)]
    pub bins_json: String,
    #[serde(default)]
    pub collection_schedule: String,
}

impl UpdateConfigArgs {
    fn resolve_bins(&self) -> Vec<BinRow> {
        if !self.bins.is_empty() {
            return self
                .bins
                .iter()
                .enumerate()
                .filter_map(|(index, input)| bin_from_input(input, index))
                .collect();
        }
        let raw = self.bins_json.trim();
        if raw.is_empty() {
            return Vec::new();
        }
        serde_json::from_str::<Vec<BinRow>>(raw)
            .unwrap_or_default()
            .into_iter()
            .filter(|b| !b.id.trim().is_empty())
            .collect()
    }
}

fn bin_from_input(input: &BinInput, index: usize) -> Option<BinRow> {
    if input.title_fr.trim().is_empty() && input.title_en.trim().is_empty() {
        return None;
    }
    let items = if input.items_fr.trim().is_empty() {
        Vec::new()
    } else {
        vec![Localized {
            fr: input.items_fr.trim().to_string(),
            en: String::new(),
        }]
    };
    Some(BinRow {
        id: format!("bin-{}", index + 1),
        title: Localized {
            fr: input.title_fr.trim().to_string(),
            en: input.title_en.trim().to_string(),
        },
        items,
        color: color_name_to_hex(&input.color),
    })
}

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(_ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    save_config(&ModuleConfig {
        bins: args.resolve_bins(),
        bins_json: String::new(),
        collection_schedule: args.collection_schedule,
    })
}
