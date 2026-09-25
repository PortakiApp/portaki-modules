//! Typed surface / operation catalogs for this module.

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOST_MAIN = "main",
    // property-stats-card / property-stats-detail pathSegment
    HOST_STATS = "calendar-sync",
}

define_operation_names! {
    LIST_SOURCES = "listSources",
    APPLY_FEEDS = "applyFeeds",
    STATS_SUMMARY = "statsSummary",
}

/// Catalog module id — kept for SDUI action builders.
#[allow(dead_code)]
pub fn module_id() -> ModuleId {
    ModuleId::from_static("ical-sync")
}
