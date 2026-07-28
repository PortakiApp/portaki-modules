//! Typed surface / operation catalogs for this module.

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOME_CARD = "home.card",
    UPCOMING_CARD = "upcoming.card",
    EXPLORE_DETAIL = "explore.detail",
}

/// Catalog module id (`train`).
#[allow(dead_code)]
pub fn module_id() -> ModuleId {
    ModuleId::from_static("train")
}
