//! Persistent entities declared for Atlas migrations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// One checklist of the property: a guest list ticked in the booklet, or a host list that
/// becomes a dated task around each stay.
///
/// Wire values of `audience`, `trigger`, `placement` and `deadline` live in [`crate::lists`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 3)]
pub struct Checklist {
    pub id: Uuid,
    pub name_fr: String,
    pub name_en: String,
    pub audience: String,
    pub icon: String,
    pub trigger: String,
    /// Guest lists only: `booklet` or `booklet+email`.
    pub placement: String,
    /// Host lists only, free text (not necessarily a workspace member).
    pub assignee_name: Option<String>,
    pub assignee_role: Option<String>,
    /// Host lists only.
    pub deadline: Option<String>,
    pub notify_assignee: bool,
    pub alert_host: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

#[portaki_sdk::entity_indexes(Checklist)]
#[allow(dead_code)]
pub const CHECKLIST_INDEXES: &[&str] = &["sort_order"];

/// One task of a checklist.
///
/// `label_fr` holds the JSON map of every language (`{"fr": …, "en": …}`), `label_en` the legacy
/// English label — see [`crate::labels`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 3)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub checklist_id: Uuid,
    pub label_fr: String,
    pub label_en: String,
    #[serde(default)]
    pub photo_required: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

#[portaki_sdk::entity_indexes(ChecklistItem)]
#[allow(dead_code)]
pub const CHECKLIST_ITEM_INDEXES: &[&str] = &["checklist_id", "sort_order"];

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
#[allow(dead_code)]
pub const CHECKLIST_COMPLETION_INDEXES: &[&str] = &["stay_id", "item_id"];

/// Ticked state of one item of a host task. Tasks are computed from the stays; only this is
/// stored.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[portaki_sdk::entity(schema_version = 3)]
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
#[allow(dead_code)]
pub const TASK_ITEM_STATE_INDEXES: &[&str] = &["task_id"];
