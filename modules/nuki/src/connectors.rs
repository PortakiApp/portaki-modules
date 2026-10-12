//! Nuki Cloud connector — remote unlock, per-stay keypad codes and lock health via host egress
//! (Bearer).
//!
//! The platform catalogue (`contracts/connectors/nuki.json`) pins every operation and binds
//! `smartlockId` to the install's `smartlock_id`. Delete is one authorization at a time, and the
//! catalogue cannot see its name: `stay_code` only ever deletes its own expired codes. No update.

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

    /// Deletes one authorization by its Nuki `id` — only expired `Portaki …` codes, see `stay_code`.
    #[portaki_sdk::connector_op(method = "DELETE", path = "/smartlock/{smartlockId}/auth/{authId}")]
    pub fn delete_auth() {}

    /// The configured lock: online state and battery, read by `tasks` (the rest is ignored).
    #[portaki_sdk::connector_op(method = "GET", path = "/smartlock/{smartlockId}")]
    pub fn get_smartlock() {}
}
