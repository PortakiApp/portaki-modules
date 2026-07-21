//! Module commands — configuration persistence.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{color_name_to_hex, load_config, save_config, BinRow, Localized, ModuleConfig};

/// One bin row from the host form (`bins.N.*`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BinInput {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub title_fr: String,
    #[serde(default)]
    pub title_en: String,
    #[serde(default)]
    pub items: String,
    #[serde(default)]
    pub items_fr: String,
    #[serde(default)]
    pub color: String,
}

/// Arguments for `updateConfig`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigArgs {
    #[serde(default)]
    pub bins: Vec<BinInput>,
    #[serde(default)]
    pub bins_json: String,
    #[serde(default)]
    pub collection_schedule: String,
}

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    let lang = Localized::lang_code(&ctx.locale);
    let existing = load_config().unwrap_or_default();
    let bins = resolve_bins(&args, &existing.bins, &lang);
    let mut collection_schedule = existing.collection_schedule;
    collection_schedule.set(&lang, args.collection_schedule.trim().to_string());
    save_config(&ModuleConfig {
        bins,
        bins_json: String::new(),
        collection_schedule,
    })
}

fn resolve_bins(args: &UpdateConfigArgs, existing: &[BinRow], lang: &str) -> Vec<BinRow> {
    if !args.bins.is_empty() {
        return args
            .bins
            .iter()
            .enumerate()
            .filter_map(|(index, input)| merge_bin(input, existing.get(index), index, lang))
            .collect();
    }
    let raw = args.bins_json.trim();
    if raw.is_empty() {
        return Vec::new();
    }
    serde_json::from_str::<Vec<BinRow>>(raw)
        .unwrap_or_default()
        .into_iter()
        .filter(|b| !b.id.trim().is_empty())
        .collect()
}

fn merge_bin(
    input: &BinInput,
    previous: Option<&BinRow>,
    index: usize,
    lang: &str,
) -> Option<BinRow> {
    let mut title = previous.map(|p| p.title.clone()).unwrap_or_default();
    let single = input.title.trim();
    if !single.is_empty() {
        title.set(lang, single.to_string());
    } else {
        if !input.title_fr.trim().is_empty() {
            title.set("fr", input.title_fr.trim().to_string());
        }
        if !input.title_en.trim().is_empty() {
            title.set("en", input.title_en.trim().to_string());
        }
    }
    if title.is_empty() {
        return None;
    }

    let items_raw = if !input.items.trim().is_empty() {
        input.items.trim()
    } else {
        input.items_fr.trim()
    };
    let mut items = previous.map(|p| p.items.clone()).unwrap_or_default();
    if items.is_empty() {
        items.push(Localized::default());
    }
    items[0].set(lang, items_raw.to_string());
    if items[0].is_empty() {
        items.clear();
    }

    Some(BinRow {
        id: previous
            .map(|p| p.id.clone())
            .unwrap_or_else(|| format!("bin-{}", index + 1)),
        title,
        items,
        color: color_name_to_hex(&input.color).or_else(|| previous.and_then(|p| p.color.clone())),
    })
}
