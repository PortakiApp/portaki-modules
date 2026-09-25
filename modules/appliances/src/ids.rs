//! Event types and catalog id for this module.

use portaki_sdk::prelude::*;

define_event_types! {
    OPEN_HOST_CHAT = "openHostChat",
}

/// Catalog module id (`appliances`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("appliances")
}
