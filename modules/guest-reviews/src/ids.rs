//! Event types and catalog id for this module.

use portaki_sdk::prelude::*;

/// Catalog module id (`guest-reviews`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("guest-reviews")
}
