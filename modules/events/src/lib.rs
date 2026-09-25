//! Portaki events module — local happenings near the property.

mod commands;
mod config;
mod connectors;
mod email_context;
mod guest;
mod host;
mod ids;
mod map_markers;
mod nearby;
mod time_format;

pub use commands::refresh_nearby;
pub use config::ModuleConfig;
pub use email_context::{email_context, EmailContextArgs, EmailContextResponse};
pub use guest::{render_explore_detail, render_home_card, render_upcoming_card};
pub use host::render_host_main;
pub use map_markers::{map_markers, MapMarkersResponse, MAX_MARKERS};
pub use nearby::{has_open_agenda, invalidate_nearby_cache, resolve_events};

portaki_sdk::portaki_module!(
    id = "events",
    display_name_key = "module.displayName",
    description_key = "module.catalogDescription",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::Calendar,
    maturity = Maturity::Beta,
    sort_order = 210,
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
