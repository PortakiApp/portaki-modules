//! Typed surface / operation catalogs for this module.

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOME_CARD = "home.card",
    HOST_MAIN = "main",
    HOST_STAY = "stay",
    HOST_STATS = "stock",
}

define_operation_names! {
    LIST_ITEMS = "listItems",
    LIST_FOR_STAY = "listForStay",
    LIST_RECENT = "listRecent",
    LIST_OPEN_COUNT = "listOpenCount",
    REPLACE_ITEMS = "replaceItems",
    SEED_DEFAULTS = "seedDefaults",
    SUBMIT = "submit",
    UPDATE_CONFIG = "updateConfig",
    UPDATE_STATUS = "updateStatus",
}

/// Catalog module id (`consumables`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("consumables")
}
