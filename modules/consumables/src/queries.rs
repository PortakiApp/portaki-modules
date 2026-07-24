//! Module queries — catalog, stay reports, open count.

use chrono::{DateTime, Utc};
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::{ConsumableItem, ConsumableReport};
use crate::storage;

/// Public item DTO returned by `listItems`.
#[portaki_sdk::wire]
#[derive(PartialEq, Eq)]
pub struct ConsumableItemDto {
    pub id: Uuid,
    pub label_fr: String,
    pub label_en: String,
    pub sort_order: i32,
    pub low_threshold: i32,
}

impl From<ConsumableItem> for ConsumableItemDto {
    fn from(value: ConsumableItem) -> Self {
        Self {
            id: value.id,
            label_fr: value.label_fr,
            label_en: value.label_en,
            sort_order: value.sort_order,
            low_threshold: value.low_threshold,
        }
    }
}

/// Row returned by report list queries.
#[portaki_sdk::wire]
#[derive(PartialEq, Eq)]
pub struct ConsumableReportRow {
    pub id: Uuid,
    pub stay_id: Uuid,
    pub item_id: Uuid,
    pub item_label: String,
    pub level: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl From<ConsumableReport> for ConsumableReportRow {
    fn from(row: ConsumableReport) -> Self {
        Self {
            id: row.id,
            stay_id: row.stay_id,
            item_id: row.item_id,
            item_label: row.item_label,
            level: row.level,
            note: row.note,
            status: row.status,
            created_at: row.created_at,
        }
    }
}

#[portaki_sdk::query(name = "listItems")]
pub fn list_items(_ctx: Context) -> Result<Vec<ConsumableItemDto>> {
    Ok(storage::list_items()?
        .into_iter()
        .map(ConsumableItemDto::from)
        .collect())
}

/// Optional host override — guest sessions ignore and use the guest stay id.
#[portaki_sdk::wire]
#[derive(Default)]
pub struct ListForStayArgs {
    #[serde(default)]
    pub stay_id: Option<Uuid>,
}

#[portaki_sdk::query(name = "listForStay")]
pub fn list_for_stay(ctx: Context, args: ListForStayArgs) -> Result<Vec<ConsumableReportRow>> {
    let stay_id = resolve_list_stay_id(&ctx, args.stay_id)?;
    Ok(storage::list_by_stay(stay_id)?
        .into_iter()
        .map(ConsumableReportRow::from)
        .collect())
}

#[portaki_sdk::query(name = "listRecent")]
pub fn list_recent(_ctx: Context) -> Result<Vec<ConsumableReportRow>> {
    Ok(storage::list_recent()?
        .into_iter()
        .map(ConsumableReportRow::from)
        .collect())
}

/// Open report count for stats cards.
#[portaki_sdk::wire]
#[derive(PartialEq, Eq)]
pub struct OpenCountDto {
    pub open_count: u32,
    pub catalog_count: u32,
}

#[portaki_sdk::query(name = "listOpenCount")]
pub fn list_open_count(_ctx: Context) -> Result<OpenCountDto> {
    Ok(OpenCountDto {
        open_count: storage::count_open()? as u32,
        catalog_count: storage::list_items()?.len() as u32,
    })
}

fn resolve_list_stay_id(ctx: &Context, stay_id: Option<Uuid>) -> Result<Uuid> {
    if let Some(guest) = ctx.guest.as_ref() {
        return Ok(guest.session_id);
    }
    stay_id.ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))
}
