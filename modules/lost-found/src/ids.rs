//! Event types and catalog id for this module.

use portaki_sdk::prelude::*;

define_event_types! {
    WORKSPACE_ACTIVITY_RECORD = "workspace-activity.record",
    // Dashboard: open the stats detail of one row in a modal.
    HOST_SURFACE_OVERLAY = "host.surface.overlay",
}

/// Catalog module id (`lost-found`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("lost-found")
}
