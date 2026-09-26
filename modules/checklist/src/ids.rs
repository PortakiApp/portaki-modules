//! Event types and catalog id for this module.

use portaki_sdk::prelude::*;

define_event_types! {
    PROGRESS_UPDATED = "checklist.progress-updated",
    COMPLETED = "checklist.completed",
    WORKSPACE_ACTIVITY_RECORD = "workspace-activity.record",
    // In this module's own namespace, which the runtime reserves to it: `consumables.*` belongs to
    // another module.
    CONSUMABLES_RESTOCKED = "checklist.consumables-restocked",
}

/// Catalog module id (`checklist`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("checklist")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restock_is_emitted_in_the_checklist_namespace() {
        assert!(CONSUMABLES_RESTOCKED.as_str().starts_with("checklist."));
    }
}
