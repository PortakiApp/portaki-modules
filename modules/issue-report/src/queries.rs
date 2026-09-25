//! Module queries — stay reports and host recent list.

use chrono::{DateTime, Utc};
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::storage;

/// Row returned by list queries.
#[portaki_sdk::wire]
#[derive(PartialEq, Eq)]
pub struct IssueReportRow {
    pub id: Uuid,
    pub stay_id: Uuid,
    pub category: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo: Option<String>,
}

impl From<crate::entities::IssueReport> for IssueReportRow {
    fn from(row: crate::entities::IssueReport) -> Self {
        Self {
            id: row.id,
            stay_id: row.stay_id,
            category: row.category,
            summary: row.summary,
            details: row.details,
            created_at: row.created_at,
            resolved_at: row.resolved_at,
            photo: row.photo,
        }
    }
}

#[portaki_sdk::query(name = "listForStay", example(label = "Signalements du séjour"))]
pub fn list_for_stay(ctx: Context) -> Result<Vec<IssueReportRow>> {
    let stay_id = require_stay_id(&ctx)?;
    Ok(storage::list_by_stay(stay_id)?
        .into_iter()
        .map(IssueReportRow::from)
        .collect())
}

#[portaki_sdk::query(name = "listRecent", example(label = "Derniers signalements"))]
pub fn list_recent(_ctx: Context) -> Result<Vec<IssueReportRow>> {
    Ok(storage::list_recent()?
        .into_iter()
        .map(IssueReportRow::from)
        .collect())
}

fn require_stay_id(ctx: &Context) -> Result<Uuid> {
    ctx.guest
        .as_ref()
        .map(|guest| guest.session_id)
        .ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))
}
