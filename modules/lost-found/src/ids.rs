//! Typed surface / operation catalogs for this module.

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOME_CARD = "home.card",
    POST_STAY_CARD = "post-stay.card",
    GUEST_FORM = "guest.form",
    HOST_STAY = "stay",
    HOST_CREATE = "create",
    // property-stats-card / property-stats-detail pathSegment
    HOST_STATS = "lost-stats",
}

define_operation_names! {
    LIST_FOR_STAY = "listForStay",
    LIST_RECENT = "listRecent",
    SUBMIT = "submit",
    SUBMIT_FOUND = "submitFound",
    UPDATE_STATUS = "updateStatus",
    EMAIL_CONTEXT = "emailContext",
    SEND_CHECKOUT_FOLLOW_UP = "sendCheckoutFollowUp",
    STATS_SUMMARY = "statsSummary",
}

define_event_types! {
    WORKSPACE_ACTIVITY_RECORD = "workspace-activity.record",
    // Dashboard: open the stats detail of one row in a modal.
    HOST_SURFACE_OVERLAY = "host.surface.overlay",
}

/// Catalog module id (`lost-found`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("lost-found")
}
