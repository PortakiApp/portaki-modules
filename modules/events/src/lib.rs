//! Portaki events module — local happenings near the property.

mod commands;
mod config;
mod connectors;
mod email_context;
mod guest;
mod host;
mod ids;
mod nearby;
mod queries;
mod time_format;

pub use commands::{refresh_nearby, update_config, EventInput, UpdateConfigArgs};
pub use config::{load_config, ModuleConfig};
pub use email_context::{email_context, EmailContextArgs, EmailContextResponse};
pub use guest::{render_explore_detail, render_home_card};
pub use host::render_host_main;
pub use nearby::{has_open_agenda, invalidate_nearby_cache, resolve_events};
pub use queries::get_config;

portaki_sdk::portaki_module!(
    id = "events",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Portaki",
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";

#[portaki_sdk::capability(
    optional,
    id = "external.open-agenda.pool",
    purpose_key = "capability.openAgenda.purpose",
    fallback_key = "capability.openAgenda.fallback"
)]
pub const OPEN_AGENDA_POOL: &str = "external.open-agenda.pool";

#[portaki_sdk::capability(
    optional,
    id = "external.open-agenda.byok",
    purpose_key = "capability.openAgenda.byok.purpose",
    fallback_key = "capability.openAgenda.byok.fallback"
)]
pub const OPEN_AGENDA_BYOK: &str = "external.open-agenda.byok";
