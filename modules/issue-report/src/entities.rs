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
}

#[portaki_sdk::entity_indexes(IssueReport)]
#[allow(dead_code)]
pub const ISSUE_REPORT_INDEXES: &[&str] = &["stay_id"];
