//! Typed surface / operation catalogs for this module.

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOME_CARD = "home.card",
    POST_STAY_CARD = "post-stay.card",
    HOST_MAIN = "main",
    // property-stats-detail pathSegments (same key as the stats tiles)
    STATS_CHECKLIST = "checklist",
    STATS_CLEANING = "cleaning",
}

define_operation_names! {
    COMPLETE_ITEM = "completeItem",
    CREATE_CHECKLIST = "createChecklist",
    DELETE_CHECKLIST = "deleteChecklist",
    EMAIL_CONTEXT = "emailContext",
    LIST_COMPLETIONS = "listCompletions",
    LIST_ITEMS = "listItems",
    PUBLISH_READINESS = "publishReadiness",
    STATS_SUMMARY = "statsSummary",
    TASK_COMPLETE = "taskComplete",
    TASK_TOGGLE = "taskToggle",
    TIMELINE_TASKS = "timelineTasks",
    UNCOMPLETE_ITEM = "uncompleteItem",
    UPDATE_CONFIG = "updateConfig",
}

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
