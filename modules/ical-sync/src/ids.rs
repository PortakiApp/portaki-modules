//! Typed surface / operation catalogs for this module.

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOST_MAIN = "main",
}

define_operation_names! {
    GET_CONFIG = "getConfig",
    UPDATE_CONFIG = "updateConfig",
    LIST_SOURCES = "listSources",
    APPLY_FEEDS = "applyFeeds",
}

/// Catalog module id (`ical-sync`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("ical-sync")
}
