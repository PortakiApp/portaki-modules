//! Persistent entities declared for Atlas migrations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// One guest issue report for a stay (many per stay allowed).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 2)]
pub struct IssueReport {
    pub id: Uuid,
    pub stay_id: Uuid,
    pub category: String,
    pub summary: String,
    pub details: Option<String>,
    pub created_at: DateTime<Utc>,
    /// Set when the status becomes `resolved`, cleared when it is reopened: `None` = still open.
    #[serde(default)]
    pub resolved_at: Option<DateTime<Utc>>,
    /// Guest photo, `portaki-file:<uuid>` (see `portaki_sdk::files::FileRef`); `None` = none.
    #[serde(default)]
    pub photo: Option<String>,
    /// `new` · `in_progress` · `resolved` — read through [`crate::status::of`]: rows written before
    /// the column carry an empty value.
    #[serde(default)]
    pub status: String,
    /// The guest's name at submit time, for the tab and the detail. Empty when unknown.
    #[serde(default)]
    pub guest_name: String,
    /// The stay's dates at submit time.
    #[serde(default)]
    pub stay_check_in: Option<DateTime<Utc>>,
    #[serde(default)]
    pub stay_check_out: Option<DateTime<Utc>>,
}

#[portaki_sdk::entity_indexes(IssueReport)]
#[allow(dead_code)]
pub const ISSUE_REPORT_INDEXES: &[&str] = &["stay_id"];

/// One line of a report's history: sent, status changed, reply sent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 2)]
pub struct IssueReportEvent {
    pub id: Uuid,
    pub report_id: Uuid,
    /// The report's stay, so the booklet reads a stay's history in one query.
    pub stay_id: Uuid,
    /// `created` · `status` · `reply`.
    pub kind: String,
    /// `guest` · `host`.
    pub actor: String,
    /// The new status of a `status` line; empty otherwise.
    #[serde(default)]
    pub status: String,
    /// The text of a `reply` line.
    #[serde(default)]
    pub text: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[portaki_sdk::entity_indexes(IssueReportEvent)]
#[allow(dead_code)]
pub const ISSUE_REPORT_EVENT_INDEXES: &[&str] = &["report_id", "stay_id"];
