//! Persistent entities declared for Atlas migrations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// One lost/found report for a stay (many per stay allowed).
///
/// `item_description` may be plain text (guest) or TipTap JSON (host-found).
/// `status` tracks host workflow — see [`crate::status`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 3)]
pub struct LostFoundReport {
    pub id: Uuid,
    pub stay_id: Uuid,
    pub kind: String,
    pub item_description: String,
    pub contact_hint: Option<String>,
    pub details: Option<String>,
    /// Wire: `to_collect` | `sent` | `returned` — default `to_collect`.
    #[serde(default = "default_status")]
    pub status: String,
    pub created_at: DateTime<Utc>,
}

fn default_status() -> String {
    crate::status::DEFAULT.to_string()
}

#[portaki_sdk::entity_indexes(LostFoundReport)]
#[allow(dead_code)]
pub const LOST_FOUND_REPORT_INDEXES: &[&str] = &["stay_id"];
