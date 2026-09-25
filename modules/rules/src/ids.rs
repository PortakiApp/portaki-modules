//! Typed surface / operation catalogs for this module.
#![allow(deprecated)]

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOME_CARD = "home.card",
    EXPLORE_DETAIL = "explore.detail",
    HOST_MAIN = "main",
}

define_operation_names! {
    EMAIL_CONTEXT = "emailContext",
    GET_CONTENT = "getContent",
    PUBLISH_READINESS = "publishReadiness",
    SAVE_CONTENT = "saveContent",
    UPDATE_CONFIG = "updateConfig",
}

/// Catalog module id (`rules`).
#[allow(dead_code)] // reserved for typed command actions (workspace Save uses updateConfig by name)
pub fn module_id() -> ModuleId {
    ModuleId::from_static("rules")
}
