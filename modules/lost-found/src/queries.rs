//! Module queries — stay reports, host recent list, email context.

use chrono::{DateTime, Utc};
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::storage;

/// Row returned by list queries.
#[portaki_sdk::wire]
#[derive(PartialEq, Eq)]
pub struct LostFoundReportRow {
    pub id: Uuid,
    pub stay_id: Uuid,
    pub kind: String,
    pub item_description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl From<crate::entities::LostFoundReport> for LostFoundReportRow {
    fn from(row: crate::entities::LostFoundReport) -> Self {
        Self {
            id: row.id,
            stay_id: row.stay_id,
            kind: row.kind,
            item_description: row.item_description,
            contact_hint: row.contact_hint,
            details: row.details,
            status: row.status,
            created_at: row.created_at,
        }
    }
}

/// Optional host override — guest sessions ignore and use the guest stay id.
#[portaki_sdk::wire]
#[portaki_sdk::params]
#[derive(Default)]
pub struct ListForStayArgs {
    #[serde(default)]
    pub stay_id: Option<Uuid>,
}

#[portaki_sdk::query(name = "listForStay")]
pub fn list_for_stay(ctx: Context, args: ListForStayArgs) -> Result<Vec<LostFoundReportRow>> {
    let stay_id = resolve_list_stay_id(&ctx, args.stay_id)?;
    Ok(storage::list_by_stay(stay_id)?
        .into_iter()
        .map(LostFoundReportRow::from)
        .collect())
}

#[portaki_sdk::query(name = "listRecent")]
pub fn list_recent(_ctx: Context) -> Result<Vec<LostFoundReportRow>> {
    Ok(storage::list_recent()?
        .into_iter()
        .map(LostFoundReportRow::from)
        .collect())
}

fn resolve_list_stay_id(ctx: &Context, stay_id: Option<Uuid>) -> Result<Uuid> {
    if let Some(guest) = ctx.guest.as_ref() {
        return Ok(guest.session_id);
    }
    stay_id.ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))
}
