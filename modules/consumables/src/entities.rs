//! Persistent entities declared for Atlas migrations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Property-scoped consumable catalog entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 1)]
pub struct ConsumableItem {
    pub id: Uuid,
    pub label_fr: String,
    pub label_en: String,
    pub sort_order: i32,
    /// Reserved for future quantity alerts — 0 means unused in v0.1 UI.
    #[serde(default)]
    pub low_threshold: i32,
    pub created_at: DateTime<Utc>,
}

#[portaki_sdk::entity_indexes(ConsumableItem)]
#[allow(dead_code)]
pub const CONSUMABLE_ITEM_INDEXES: &[&str] = &["sort_order"];

/// Stay-scoped guest shortage report (many per stay).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 1)]
pub struct ConsumableReport {
    pub id: Uuid,
    pub stay_id: Uuid,
    pub item_id: Uuid,
    /// Label snapshot at submit time (catalog may change later).
    pub item_label: String,
    /// Wire: `missing` | `low`.
    pub level: String,
    pub note: Option<String>,
    /// Wire: `open` | `restocked` — default `open`.
    #[serde(default = "default_status")]
    pub status: String,
    pub created_at: DateTime<Utc>,
}

fn default_status() -> String {
    crate::status::DEFAULT.to_string()
}

#[portaki_sdk::entity_indexes(ConsumableReport)]
#[allow(dead_code)]
pub const CONSUMABLE_REPORT_INDEXES: &[&str] = &["stay_id", "status"];
