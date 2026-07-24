//! Typed surface / operation catalogs for this module.

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOME_CARD = "home.card",
    HOST_MAIN = "main",
    HOST_STAY = "stay",
    HOST_CREATE = "create",
}

define_operation_names! {
    LIST_FOR_STAY = "listForStay",
    LIST_RECENT = "listRecent",
    SUBMIT = "submit",
    SUBMIT_FOUND = "submitFound",
    UPDATE_STATUS = "updateStatus",
    UPDATE_CONFIG = "updateConfig",
    EMAIL_CONTEXT = "emailContext",
    SEND_CHECKOUT_FOLLOW_UP = "sendCheckoutFollowUp",
}

// Domain events removed — transactional mail uses host::email::send.

/// Catalog module id (`lost-found`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("lost-found")
}
