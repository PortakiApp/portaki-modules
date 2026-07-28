//! Typed surface / operation catalogs for this module.

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOME_CARD = "home.card",
    UPCOMING_CARD = "upcoming.card",
    EXPLORE_DETAIL = "explore.detail",
    HOST_MAIN = "main",
}

define_operation_names! {
    EMAIL_CONTEXT = "emailContext",
    GET_CONFIG = "getConfig",
    UPDATE_CONFIG = "updateConfig",
}

/// Catalog module id (`access-guide`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("access-guide")
}
