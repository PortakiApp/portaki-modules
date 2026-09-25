//! Event types and catalog id for this module.

use portaki_sdk::prelude::*;

define_event_types! {
    COMPLETED = "pre-arrival.completed",
}

/// Catalog module id (`pre-arrival-form`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("pre-arrival-form")
}
