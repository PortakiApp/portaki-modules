//! Module commands — host replace + guest complete / uncomplete.

use portaki_sdk::host::events;
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::storage;

/// Single item payload for `replaceItems`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItemInput {
    #[serde(default, alias = "labelFr")]
    pub label_fr: String,
    #[serde(default, alias = "labelEn")]
    pub label_en: String,
    #[serde(default, alias = "sortOrder")]
    pub sort_order: i32,
}

/// Arguments for `replaceItems`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceItemsArgs {
    /// Structured items array (preferred).
    #[serde(default)]
    pub items: Vec<ChecklistItemInput>,
    /// Optional JSON string of items (legacy host TextArea).
    #[serde(default, alias = "itemsJson")]
    pub items_json: Option<String>,
}

/// Arguments for complete / uncomplete.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemIdArgs {
    pub item_id: Uuid,
}

impl ReplaceItemsArgs {
    fn resolve_items(&self) -> Result<Vec<ChecklistItemInput>> {
        let from_array: Vec<ChecklistItemInput> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                if item.label_fr.trim().is_empty() && item.label_en.trim().is_empty() {
                    return None;
                }
                Some(ChecklistItemInput {
                    label_fr: item.label_fr.trim().to_string(),
                    label_en: item.label_en.trim().to_string(),
                    sort_order: if item.sort_order == 0 {
                        index as i32
                    } else {
                        item.sort_order
                    },
                })
            })
            .collect();
        if !from_array.is_empty() || self.items_json.is_none() {
            return Ok(from_array);
        }
        let Some(raw) = self.items_json.as_ref() else {
            return Ok(Vec::new());
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }
        serde_json::from_str(trimmed)
            .map_err(|error| PortakiError::Host(format!("invalid items_json: {error}")))
    }
}

#[portaki_sdk::command(name = "replaceItems")]
pub fn replace_items(_ctx: Context, args: ReplaceItemsArgs) -> Result<()> {
    let items = args.resolve_items()?;
    let payload = items
        .into_iter()
        .map(|item| (item.label_fr, item.label_en, item.sort_order))
        .collect();
    storage::replace_items(payload)
}

#[portaki_sdk::command(name = "completeItem")]
pub fn complete_item(ctx: Context, args: ItemIdArgs) -> Result<()> {
    let stay_id = require_stay_id(&ctx)?;
    storage::complete_item(stay_id, args.item_id)?;
    emit_progress(stay_id)
}

#[portaki_sdk::command(name = "uncompleteItem")]
pub fn uncomplete_item(ctx: Context, args: ItemIdArgs) -> Result<()> {
    let stay_id = require_stay_id(&ctx)?;
    storage::uncomplete_item(stay_id, args.item_id)?;
    emit_progress(stay_id)
}

fn require_stay_id(ctx: &Context) -> Result<Uuid> {
    ctx.guest
        .as_ref()
        .map(|guest| guest.session_id)
        .ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))
}

fn emit_progress(stay_id: Uuid) -> Result<()> {
    let percentage = storage::progress_percent(stay_id)?;
    let _ = events::emit(
        "checklist.progress-updated",
        &json!({ "percentage": percentage }),
    );
    if percentage == 100 {
        let _ = events::emit("checklist.completed", &json!({ "stayId": stay_id }));
    }
    Ok(())
}
