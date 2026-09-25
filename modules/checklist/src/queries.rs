//! Module queries — guest items and stay completions.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::ChecklistItem;
use crate::i18n::text;
use crate::lists;
use crate::storage;

/// Public item DTO returned by `listItems`.
#[portaki_sdk::wire]
#[derive(PartialEq, Eq)]
pub struct ChecklistItemDto {
    pub id: Uuid,
    pub checklist_id: Uuid,
    pub label_fr: String,
    pub label_en: String,
    pub sort_order: i32,
}

impl From<ChecklistItem> for ChecklistItemDto {
    fn from(value: ChecklistItem) -> Self {
        Self {
            id: value.id,
            checklist_id: value.checklist_id,
            label_fr: value.label_fr,
            label_en: value.label_en,
            sort_order: value.sort_order,
        }
    }
}

/// Items of every guest list, list by list.
pub fn guest_items() -> Result<Vec<ChecklistItem>> {
    let mut items = Vec::new();
    for list in storage::list_checklists()? {
        if list.audience == lists::GUEST {
            items.extend(storage::items_of(list.id)?);
        }
    }
    Ok(items)
}

#[portaki_sdk::query(name = "listItems")]
pub fn list_items(_ctx: Context) -> Result<Vec<ChecklistItemDto>> {
    Ok(guest_items()?
        .into_iter()
        .map(ChecklistItemDto::from)
        .collect())
}

#[portaki_sdk::query(name = "listCompletions")]
pub fn list_completions(ctx: Context) -> Result<Vec<Uuid>> {
    let stay_id = ctx
        .guest
        .as_ref()
        .map(|guest| guest.session_id)
        .ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))?;
    Ok(storage::list_completions(Some(stay_id))?
        .into_iter()
        .map(|row| row.item_id)
        .collect())
}

/// Blocks publication until a list has an item: an empty list shows nothing to the guest and
/// gives the host no task.
#[portaki_sdk::query(name = "publishReadiness")]
pub fn publish_readiness(_ctx: Context) -> Result<PublishReadiness> {
    let ok = !storage::list_items()?.is_empty();
    Ok(PublishReadiness {
        items: vec![PublishCheck {
            id: "checklist".into(),
            level: PublishLevel::Required,
            ok,
            label: text("publish.checklist.label", &[]),
            hint: text("publish.checklist.hint", &[]),
        }],
    })
}
