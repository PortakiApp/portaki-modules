//! Typed surface / operation catalogs for this module.

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOME_CARD = "home.card",
    HOST_MAIN = "main",
    HOST_STAY = "stay",
}

define_operation_names! {
    GET_STATUS = "getStatus",
    SUBMIT = "submit",
    UPDATE_CONFIG = "updateConfig",
}

define_event_types! {
    COMPLETED = "pre-arrival.completed",
}

/// Catalog module id (`pre-arrival-form`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("pre-arrival-form")
}
