//! Typed surface / operation catalogs for this module.

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOME_CARD = "home.card",
    GUEST_FORM = "guest.form",
    HOST_MAIN = "main",
}

define_operation_names! {
    LIST_FOR_STAY = "listForStay",
    LIST_RECENT = "listRecent",
    SUBMIT = "submit",
}

/// Catalog module id (`issue-report`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("issue-report")
}
