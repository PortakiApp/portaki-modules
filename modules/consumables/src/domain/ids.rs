//! Event types and catalog id for this module.

use portaki_sdk::prelude::*;

/// Catalog module id (`consumables`).
pub fn module_id() -> ModuleId {
    ModuleId::from_static("consumables")
}
