//! Portaki local-guide module — nearby spots and host picks.

mod activities;
mod affiliate;
mod config;
mod email_context;
mod guest;
mod host;
mod ids;
mod map_markers;
mod tiqets;

pub use affiliate::{
    normalize_curated_url, search_url, CuratedUrlError, MAX_CURATED_LINKS, PARTNER_ID,
    PARTNER_QUERY_PARAM,
};
pub use config::{ActivitiesConfig, ActivityRow, ModuleConfig, TiqetsConfig};
pub use email_context::{email_context, EmailContextArgs, EmailContextResponse};
pub use guest::{render_explore_detail, render_home_card, render_upcoming_card};
pub use host::render_host_main;
pub use map_markers::{map_markers, MapMarkersResponse, MAX_MARKERS};
pub use tiqets::{has_tiqets, FRESH_SECS, MAX_PRODUCTS, STALE_MAX_SECS};

portaki_sdk::portaki_module!(
    id = "local-guide",
    display_name_key = "module.catalogName",
    description_key = "module.catalogDescription",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::MapPin,
    maturity = Maturity::Stable,
    sort_order = 140,
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";

#[portaki_sdk::capability(
    optional,
    id = "external.tiqets.pool",
    purpose_key = "capability.tiqets.purpose",
    fallback_key = "capability.tiqets.fallback"
)]
pub const TIQETS_POOL: &str = "external.tiqets.pool";

#[portaki_sdk::capability(
    optional,
    id = "external.tiqets.byok",
    purpose_key = "capability.tiqets.byok.purpose",
    fallback_key = "capability.tiqets.byok.fallback"
)]
pub const TIQETS_BYOK: &str = "external.tiqets.byok";
