//! Module commands — save house rules content.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::{RuleItem, RulesPayload};
use crate::store;

/// One bilingual rule row from the host form (`items.N.*`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleItemInput {
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub title_fr: String,
    #[serde(default)]
    pub subtitle_fr: String,
    #[serde(default)]
    pub title_en: String,
    #[serde(default)]
    pub subtitle_en: String,
}

/// Arguments for `saveContent`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveContentArgs {
    /// Structured bilingual items (preferred).
    #[serde(default)]
    pub items: Vec<RuleItemInput>,
    /// Legacy JSON string for French payload.
    #[serde(default)]
    pub content_fr: String,
    /// Legacy JSON string for English payload.
    #[serde(default)]
    pub content_en: String,
}

#[portaki_sdk::command(name = "saveContent")]
pub fn save_content(_ctx: Context, args: SaveContentArgs) -> Result<()> {
    let (fr, en) = if !args.items.is_empty() {
        build_payloads_from_items(&args.items)
    } else {
        (
            RulesPayload::parse(&args.content_fr),
            RulesPayload::parse(&args.content_en),
        )
    };
    let fr = fr
        .to_json_string()
        .map_err(|e| PortakiError::Host(format!("content_fr: {e}")))?;
    let en = en
        .to_json_string()
        .map_err(|e| PortakiError::Host(format!("content_en: {e}")))?;
    let _ = store::save_content_row(fr, en)?;
    Ok(())
}

fn build_payloads_from_items(items: &[RuleItemInput]) -> (RulesPayload, RulesPayload) {
    let mut fr_items = Vec::new();
    let mut en_items = Vec::new();
    for item in items {
        if item.title_fr.trim().is_empty() && item.title_en.trim().is_empty() {
            continue;
        }
        let icon = item.icon.trim().to_string();
        fr_items.push(RuleItem {
            icon: icon.clone(),
            title: item.title_fr.trim().to_string(),
            subtitle: item.subtitle_fr.trim().to_string(),
        });
        en_items.push(RuleItem {
            icon,
            title: item.title_en.trim().to_string(),
            subtitle: item.subtitle_en.trim().to_string(),
        });
    }
    (
        RulesPayload { items: fr_items },
        RulesPayload { items: en_items },
    )
}
