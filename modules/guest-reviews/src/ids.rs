//! Typed surface / operation catalogs for this module.
#![allow(deprecated)]

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOME_CARD = "home.card",
    POST_STAY_CARD = "post-stay.card",
    HOST_MAIN = "main",
    // property-stats-card / property-stats-detail pathSegment
    HOST_STATS = "reviews",
}

define_operation_names! {
    STATS_SUMMARY = "statsSummary",
    SUBMIT_REVIEW = "submitReview",
}

/// Catalog module id (`guest-reviews`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("guest-reviews")
}
