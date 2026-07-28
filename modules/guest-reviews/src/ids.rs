//! Typed surface / operation catalogs for this module.

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOME_CARD = "home.card",
    POST_STAY_CARD = "post-stay.card",
    HOST_MAIN = "main",
}

define_operation_names! {
    GET_CONFIG = "getConfig",
    SUBMIT_REVIEW = "submitReview",
    UPDATE_CONFIG = "updateConfig",
}

/// Catalog module id (`guest-reviews`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("guest-reviews")
}
