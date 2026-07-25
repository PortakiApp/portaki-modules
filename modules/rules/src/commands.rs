//! Module commands — save house rules content.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::{RuleItem, RulesBundle, RulesPayload};
use crate::store;

/// One rule row from the host form (`items.N.*`) for the active locale.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleItemInput {
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    /// Legacy bilingual fields.
    #[serde(default)]
    pub title_fr: String,
    #[serde(default)]
    pub subtitle_fr: String,
    #[serde(default)]
    pub title_en: String,
    #[serde(default)]
    pub subtitle_en: String,
}

/// Arguments for `saveContent` / `updateConfig` (workspace Save chrome).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveContentArgs {
    /// Structured items for the active request locale (preferred).
    #[serde(default)]
    pub items: Vec<RuleItemInput>,
    /// Legacy JSON string for French payload.
    #[serde(default)]
    pub content_fr: String,
    /// Legacy JSON string for English payload.
    #[serde(default)]
    pub content_en: String,
}

/// Workspace header Save → nested form `{ items: [{ icon, title, subtitle }] }`.
///
/// Same payload as [`save_content`] — host `HostSurfacePanel` always dispatches
/// `updateConfig` for property-workspace-tab modules.
#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(ctx: Context, args: SaveContentArgs) -> Result<()> {
    save_content(ctx, args)
}

#[portaki_sdk::command(name = "saveContent")]
pub fn save_content(ctx: Context, args: SaveContentArgs) -> Result<()> {
    let lang = RulesBundle::lang_code(&ctx.locale);
    let existing = store::load_content()?;
    let (prev_fr, prev_en) = match existing {
        Some(row) => (row.content_fr, row.content_en),
        None => (String::new(), String::new()),
    };
    let mut bundle = RulesBundle::from_row(&prev_fr, &prev_en);

    if !args.items.is_empty() {
        let has_legacy_dual = args.items.iter().any(|i| {
            !i.title_fr.trim().is_empty()
                || !i.title_en.trim().is_empty()
                || !i.subtitle_fr.trim().is_empty()
                || !i.subtitle_en.trim().is_empty()
        });
        if has_legacy_dual {
            let (fr, en) = build_payloads_from_legacy_items(&args.items);
            bundle.set("fr", fr.clone());
            bundle.set("en", en);
            bundle.sync_icons_from(&fr);
        } else {
            let payload = build_payload_for_lang(&args.items);
            bundle.set(&lang, payload.clone());
            bundle.sync_icons_from(&payload);
        }
    } else if !args.content_fr.trim().is_empty() || !args.content_en.trim().is_empty() {
        // Legacy raw JSON path — seed/migrate into bundle.
        let migrated = RulesBundle::from_row(&args.content_fr, &args.content_en);
        for (k, v) in migrated.by_lang {
            bundle.set(&k, v);
        }
    }

    let json = bundle
        .to_json_string()
        .map_err(|e| PortakiError::Host(format!("content: {e}")))?;
    let _ = store::save_content_row(json, String::new())?;
    Ok(())
}

fn build_payload_for_lang(items: &[RuleItemInput]) -> RulesPayload {
    let mut out = Vec::new();
    for item in items {
        if item.title.trim().is_empty() {
            continue;
        }
        out.push(RuleItem {
            icon: item.icon.trim().to_string(),
            title: item.title.trim().to_string(),
            subtitle: item.subtitle.trim().to_string(),
        });
    }
    RulesPayload { items: out }
}

fn build_payloads_from_legacy_items(items: &[RuleItemInput]) -> (RulesPayload, RulesPayload) {
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
