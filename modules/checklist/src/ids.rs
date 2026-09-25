//! Event types and catalog id for this module.

use portaki_sdk::prelude::*;

define_event_types! {
    PROGRESS_UPDATED = "checklist.progress-updated",
    COMPLETED = "checklist.completed",
    WORKSPACE_ACTIVITY_RECORD = "workspace-activity.record",
    CONSUMABLES_RESTOCKED = "consumables.restocked",
}

/// Catalog module id (`checklist`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("checklist")
}
