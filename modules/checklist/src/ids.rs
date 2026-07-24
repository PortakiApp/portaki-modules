//! Typed surface / operation catalogs for this module.

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOME_CARD = "home.card",
    HOST_MAIN = "main",
}

define_operation_names! {
    COMPLETE_ITEM = "completeItem",
    EMAIL_CONTEXT = "emailContext",
    LIST_COMPLETIONS = "listCompletions",
    LIST_ITEMS = "listItems",
    REPLACE_ITEMS = "replaceItems",
    UNCOMPLETE_ITEM = "uncompleteItem",
    UPDATE_CONFIG = "updateConfig",
}

define_event_types! {
    PROGRESS_UPDATED = "checklist.progress-updated",
    COMPLETED = "checklist.completed",
}

/// Catalog module id (`checklist`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("checklist")
}
