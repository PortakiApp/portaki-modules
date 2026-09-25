//! Typed surface / operation catalogs for this module.

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOME_CARD = "home.card",
    EXPLORE_DETAIL = "explore.detail",
    HOST_MAIN = "main",
}

define_operation_names! {
    EMAIL_CONTEXT = "emailContext",
}

/// Catalog module id (`wifi-guest`).
#[allow(dead_code)]
pub fn module_id() -> ModuleId {
    ModuleId::from_static("wifi-guest")
}
