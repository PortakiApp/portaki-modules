//! Module commands — host catalog + guest submit + host status.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::email_send;
use crate::labels::{self, lang_code};
use crate::level;
use crate::status;
use crate::storage;

/// Single item payload for `replaceItems` / `updateConfig`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumableItemInput {
    #[serde(default)]
    pub label: String,
    #[serde(default, alias = "labelFr")]
    pub label_fr: String,
    #[serde(default, alias = "labelEn")]
    pub label_en: String,
    #[serde(default, alias = "sortOrder")]
    pub sort_order: i32,
    #[serde(default, alias = "lowThreshold")]
    pub low_threshold: i32,
}

/// Arguments for `replaceItems`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceItemsArgs {
    #[serde(default)]
    pub items: Vec<ConsumableItemInput>,
    #[serde(default, alias = "itemsJson")]
    pub items_json: Option<String>,
}

impl ReplaceItemsArgs {
    fn resolve_items(&self) -> Result<Vec<ConsumableItemInput>> {
        let from_array: Vec<ConsumableItemInput> = self
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
                Some(ConsumableItemInput {
                    label: item.label.trim().to_string(),
                    label_fr: item.label_fr.trim().to_string(),
                    label_en: item.label_en.trim().to_string(),
                    sort_order: if item.sort_order == 0 {
                        index as i32
                    } else {
                        item.sort_order
                    },
                    low_threshold: item.low_threshold.max(0),
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
    pub items: Vec<ConsumableItemInput>,
}

/// Persists catalog items from the host workspace Save chrome.
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
        let mut labels_map = existing
            .get(index)
            .map(labels::labels_from_item)
            .unwrap_or_default();
        if !input.label.trim().is_empty() {
            labels_map.insert(lang.clone(), input.label.trim().to_string());
        } else {
            if !input.label_fr.trim().is_empty() {
                labels_map.insert("fr".into(), input.label_fr.trim().to_string());
            }
            if !input.label_en.trim().is_empty() {
                labels_map.insert("en".into(), input.label_en.trim().to_string());
            }
        }
        let (label_fr, label_en) = labels::encode_labels(&labels_map);
        let id = existing.get(index).map(|item| item.id);
        let low_threshold = existing
            .get(index)
            .map(|item| item.low_threshold)
            .unwrap_or(input.low_threshold);
        next.push((
            id,
            label_fr,
            label_en,
            input.sort_order,
            if input.low_threshold > 0 {
                input.low_threshold
            } else {
                low_threshold
            },
        ));
    }
    storage::replace_items_preserving_ids(next)
}

/// Fills an empty catalog with common consumables (FR/EN). No-op if items exist.
#[portaki_sdk::command(name = "seedDefaults")]
pub fn seed_defaults(ctx: Context, _args: EmptyArgs) -> Result<()> {
    if !storage::list_items()?.is_empty() {
        return Ok(());
    }
    replace_items(
        ctx,
        ReplaceItemsArgs {
            items: default_catalog(),
            items_json: None,
        },
    )
}

fn default_catalog() -> Vec<ConsumableItemInput> {
    [
        ("Papier toilette", "Toilet paper"),
        ("Savon", "Hand soap"),
        ("Gel douche", "Shower gel"),
        ("Shampoing", "Shampoo"),
        ("Café", "Coffee"),
        ("Tablettes lave-vaisselle", "Dishwasher tablets"),
        ("Essuie-tout", "Paper towels"),
        ("Lessive", "Laundry detergent"),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (fr, en))| ConsumableItemInput {
        label: String::new(),
        label_fr: fr.into(),
        label_en: en.into(),
        sort_order: index as i32,
        low_threshold: 0,
    })
    .collect()
}

/// Arguments for guest `submit`.
#[portaki_sdk::wire]
pub struct SubmitArgs {
    pub item_id: Uuid,
    pub level: String,
    #[serde(default)]
    pub note: Option<String>,
}

#[portaki_sdk::command(name = "submit")]
pub fn submit(ctx: Context, args: SubmitArgs) -> Result<()> {
    let stay_id = require_guest_stay_id(&ctx)?;
    let level = level::parse_level(&args.level)?;
    let note = normalize_optional(args.note);

    let item = storage::find_item(args.item_id)?
        .ok_or_else(|| PortakiError::Host("item_not_found".to_string()))?;
    let item_label = labels::pick_label(
        &labels::labels_from_item(&item),
        &ctx.locale,
        &ctx.property.locale,
    );
    if item_label.trim().is_empty() {
        return Err(PortakiError::Host("item_label_empty".to_string()));
    }

    let report = storage::create_report(
        stay_id,
        item.id,
        item_label.clone(),
        level.clone(),
        note.clone(),
    )?;

    email_send::notify_host_submitted(
        ctx.property_id,
        stay_id,
        report.id,
        &item_label,
        &level,
        note.as_deref(),
    )?;
    Ok(())
}

/// Arguments for host `updateStatus`.
#[portaki_sdk::wire]
pub struct UpdateStatusArgs {
    pub report_id: Uuid,
    pub status: String,
}

#[portaki_sdk::command(name = "updateStatus")]
pub fn update_status(ctx: Context, args: UpdateStatusArgs) -> Result<()> {
    if ctx.guest.is_some() {
        return Err(PortakiError::Host("host_only".to_string()));
    }

    let status = status::parse_status(&args.status)?;
    let _ = storage::update_status(args.report_id, status)?;
    Ok(())
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn require_guest_stay_id(ctx: &Context) -> Result<Uuid> {
    ctx.guest
        .as_ref()
        .map(|guest| guest.session_id)
        .ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))
}
