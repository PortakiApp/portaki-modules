//! Persistent entities declared for Atlas migrations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stay-scoped completion of a guest checklist item.
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
#[allow(dead_code, reason = "read by the migrations, not by the code")]
pub const CHECKLIST_COMPLETION_INDEXES: &[&str] = &["stay_id", "item_id"];

/// Ticked state of one item of a host task. Tasks are computed from the stays; only this is
/// stored.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 2)]
pub struct TaskItemState {
    pub id: Uuid,
    /// `<checklistId>:<stayId>`.
    pub task_id: String,
    pub item_id: Uuid,
    pub done: bool,
    /// `portaki-file:<uuid>` attached when ticked.
    pub photo: Option<String>,
    pub done_at: Option<DateTime<Utc>>,
}

#[portaki_sdk::entity_indexes(TaskItemState)]
#[allow(dead_code, reason = "read by the migrations, not by the code")]
pub const TASK_ITEM_STATE_INDEXES: &[&str] = &["task_id"];
