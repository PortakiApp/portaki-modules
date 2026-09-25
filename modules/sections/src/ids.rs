//! Typed surface / operation catalogs for this module.
#![allow(deprecated)]

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOME_CARD = "home.card",
    EXPLORE_SHEET = "explore.sheet",
    HOST_MAIN = "main",
}

define_operation_names! {
    DELETE_SECTION = "deleteSection",
    LIST_SECTIONS = "listSections",
    REORDER = "reorder",
    SAVE_SECTION = "saveSection",
}

/// Catalog module id (`sections`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("sections")
}
