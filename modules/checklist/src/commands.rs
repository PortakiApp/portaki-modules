//! Module commands — host replace + guest complete / uncomplete.

use portaki_sdk::host::events;
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::labels::{self, lang_code};
use crate::storage;

/// Single item payload for `replaceItems`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItemInput {
    #[serde(default)]
    pub label: String,
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
#[portaki_sdk::wire]
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
                let empty = item.label.trim().is_empty()
                    && item.label_fr.trim().is_empty()
                    && item.label_en.trim().is_empty();
                if empty {
                    return None;
                }
                Some(ChecklistItemInput {
                    label: item.label.trim().to_string(),
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

/// Workspace header Save → nested form `{ items: [{ label }] }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigArgs {
    #[serde(default)]
    pub items: Vec<ChecklistItemInput>,
}

/// Persists checklist items from the host workspace Save chrome.
#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    replace_items(
        ctx,
        ReplaceItemsArgs {
            items: args.items,
            items_json: None,
        },
    )
}

#[portaki_sdk::command(name = "replaceItems")]
pub fn replace_items(ctx: Context, args: ReplaceItemsArgs) -> Result<()> {
    let lang = lang_code(&ctx.locale);
    let items = args.resolve_items()?;
    let existing = storage::list_items()?;
    let mut next = Vec::new();
    for (index, input) in items.into_iter().enumerate() {
        let mut labels = existing
            .get(index)
            .map(labels::labels_from_item)
            .unwrap_or_default();
        if !input.label.trim().is_empty() {
            labels.insert(lang.clone(), input.label.trim().to_string());
        } else {
            if !input.label_fr.trim().is_empty() {
                labels.insert("fr".into(), input.label_fr.trim().to_string());
            }
            if !input.label_en.trim().is_empty() {
                labels.insert("en".into(), input.label_en.trim().to_string());
            }
        }
        let (label_fr, label_en) = labels::encode_labels(&labels);
        let id = existing.get(index).map(|item| item.id);
        next.push((id, label_fr, label_en, input.sort_order));
    }
    storage::replace_items_preserving_ids(next)
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

#[derive(Serialize)]
struct ProgressPayload {
    percentage: u8,
}

#[portaki_sdk::wire(serialize)]
struct CompletedPayload {
    stay_id: Uuid,
}

fn emit_progress(stay_id: Uuid) -> Result<()> {
    let percentage = storage::progress_percent(stay_id)?;
    events::emit(
        crate::ids::PROGRESS_UPDATED,
        &ProgressPayload { percentage },
    )?;
    if percentage == 100 {
        events::emit(crate::ids::COMPLETED, &CompletedPayload { stay_id })?;
    }
    Ok(())
}
