//! Persistent entities declared for Atlas migrations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Property-scoped checkout checklist task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 1)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub label_fr: String,
    pub label_en: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

#[portaki_sdk::entity_indexes(ChecklistItem)]
#[allow(dead_code)]
pub const CHECKLIST_ITEM_INDEXES: &[&str] = &["sort_order"];

/// Stay-scoped completion of a checklist item.
///
/// `property_id` is injected by typed-repo from invocation context; kept in the
/// schema so SELECT/INSERT/DELETE with `WHERE property_id = …` succeed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 2)]
pub struct ChecklistCompletion {
    pub id: Uuid,
    pub stay_id: Uuid,
    pub item_id: Uuid,
    pub completed_at: DateTime<Utc>,
}

#[portaki_sdk::entity_indexes(ChecklistCompletion)]
#[allow(dead_code)]
pub const CHECKLIST_COMPLETION_INDEXES: &[&str] = &["stay_id", "item_id"];
