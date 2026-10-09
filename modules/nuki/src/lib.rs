//! Portaki Nuki module — smart-lock provider for `access.smart_lock`.
//!
//! Requires `portaki` CLI ≥ 2.1.0 (CapabilityId catalog includes `access.smart_lock`).

mod commands;
mod config;
mod connectors;
mod host;
mod i18n;
mod queries;
mod stay_code;

pub use commands::{
    get_guest_credential, unlock, GuestCredentialResponse, StayArgs, UnlockResponse,
};
pub use config::ModuleConfig;
pub use host::{render_host_main, render_host_stay};
pub use queries::publish_readiness;

portaki_sdk::portaki_module!(
    id = "nuki",
    display_name_key = "module.displayName",
    description_key = "module.catalogDescription",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::Lock,
    maturity = Maturity::Beta,
    sort_order = 230,
    feeds = "access-guide",
);

#[portaki_sdk::capability(provided, id = "access.smart_lock")]
pub const SMART_LOCK: &str = "access.smart_lock";

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";

#[portaki_sdk::capability(
    optional,
    id = "external.nuki.byok",
    purpose_key = "capability.nuki.byok.purpose",
    fallback_key = "capability.nuki.byok.fallback"
)]
pub const NUKI_BYOK: &str = "external.nuki.byok";

#[cfg(test)]
mod capability_tests {
    use portaki_sdk::capability;
    use portaki_sdk::contracts::smart_lock;

    #[test]
    fn declares_smart_lock_capability_id() {
        assert_eq!(super::SMART_LOCK, smart_lock::CAPABILITY.as_str());
    }

    /// The declared id and the one `has_nuki_byok` asks for are the same capability.
    #[test]
    fn declares_the_nuki_byok_capability_id() {
        assert_eq!(super::NUKI_BYOK, capability::external::NUKI_BYOK.as_str());
    }
}
