//! OpenAgenda connector declaration for the events module manifest.
//! HTTP paths are owned by the module; runtime executes generic egress from this metadata.

#[portaki_sdk::custom_connector(
    id = "open-agenda",
    display_name_key = "connector.openAgenda.name",
    base_url = "https://api.openagenda.com",
    credential_provider_id = "open-agenda",
    auth = "query_key"
)]
#[allow(dead_code)] // metadata-only; macros emit manifest emissions at compile time
pub struct ModuleOpenAgenda;

#[allow(dead_code)] // metadata-only; macros emit manifest emissions at compile time
impl ModuleOpenAgenda {
    #[portaki_sdk::connector_op(method = "GET", path = "/v2/events", cache = "1h")]
    pub fn nearby_events() {}
}
