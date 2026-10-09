//! Nuki Cloud connector — remote unlock and per-stay keypad codes via host egress (Bearer).
//!
//! The platform catalogue (`contracts/connectors/nuki.json`) pins every operation and binds
//! `smartlockId` to the install's `smartlock_id`. No delete, no update: a code is never revoked by
//! the module, only bounded by its `allowedUntilDate`.

#[portaki_sdk::custom_connector(
    id = "nuki",
    display_name_key = "connector.nuki.name",
    base_url = "https://api.nuki.io",
    credential_provider_id = "nuki"
)]
#[allow(dead_code)]
pub struct ModuleNuki;

#[allow(dead_code)]
impl ModuleNuki {
    #[portaki_sdk::connector_op(method = "POST", path = "/smartlock/{smartlockId}/action/unlock")]
    pub fn remote_unlock() {}

    /// The lock's authorizations — keypad codes included, with their code.
    #[portaki_sdk::connector_op(method = "GET", path = "/smartlock/{smartlockId}/auth")]
    pub fn list_auths() {}

    /// Creates an authorization; Nuki answers 204, without its id.
    #[portaki_sdk::connector_op(method = "PUT", path = "/smartlock/{smartlockId}/auth")]
    pub fn create_auth() {}
}
