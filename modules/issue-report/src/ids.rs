//! Typed surface / operation catalogs for this module.
#![allow(deprecated)]

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOME_CARD = "home.card",
    GUEST_FORM = "guest.form",
    // property-stats-card / property-stats-detail pathSegment
    HOST_STATS = "issue-stats",
}

define_operation_names! {
    LIST_FOR_STAY = "listForStay",
    LIST_RECENT = "listRecent",
    SUBMIT = "submit",
    RESOLVE = "resolve",
    STATS_SUMMARY = "statsSummary",
}

define_event_types! {
    WORKSPACE_ACTIVITY_RECORD = "workspace-activity.record",
    // Dashboard: open the stats detail of one row in a modal.
    HOST_SURFACE_OVERLAY = "host.surface.overlay",
}

/// Catalog module id (`issue-report`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("issue-report")
}
