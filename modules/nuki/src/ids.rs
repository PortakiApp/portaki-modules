//! Typed surface / operation catalogs for this module.

use portaki_sdk::prelude::*;

define_surface_ids! {
    HOST_MAIN = "main",
}

// UNLOCK / GET_GUEST_CREDENTIAL must stay aligned with
// `portaki_sdk::contracts::smart_lock` (peer protocol).
define_operation_names! {
    UNLOCK = "unlock",
    GET_GUEST_CREDENTIAL = "getGuestCredential",
}

/// Catalog module id (`nuki`).
#[allow(dead_code)]
pub fn module_id() -> ModuleId {
    ModuleId::from_static("nuki")
}

#[cfg(test)]
mod tests {
    use portaki_sdk::contracts::smart_lock;

    #[test]
    fn smart_lock_ops_match_sdk_contract() {
        assert_eq!(super::UNLOCK, smart_lock::UNLOCK);
        assert_eq!(
            super::GET_GUEST_CREDENTIAL,
            smart_lock::GET_GUEST_CREDENTIAL
        );
    }
}
