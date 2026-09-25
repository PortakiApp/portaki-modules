//! Typed surface / operation catalogs for this module.
#![allow(deprecated)]

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOME_CARD = "home.card",
    EXPLORE_DETAIL = "explore.detail",
    EXPLORE_ITEM = "explore.item",
    HOST_MAIN = "main",
}

define_operation_names! {
    DELETE_APPLIANCE = "deleteAppliance",
    GET_CONTENT = "getContent",
    REORDER_APPLIANCES = "reorderAppliances",
    REPLACE_DEVICES = "replaceDevices",
    SAVE_APPLIANCE = "saveAppliance",
    SAVE_SAFETY_NOTICE = "saveSafetyNotice",
}

define_event_types! {
    OPEN_HOST_CHAT = "openHostChat",
}

/// Catalog module id (`appliances`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("appliances")
}
