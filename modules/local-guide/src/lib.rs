//! Portaki local-guide module — nearby spots and host picks.

mod activities;
mod affiliate;
mod config;
mod email_context;
mod guest;
mod host;
mod map_markers;
mod provider;
mod tiqets;
mod viator;

pub use affiliate::{
    normalize_curated_url, search_url, CuratedUrlError, MAX_CURATED_LINKS, PARTNER_ID,
    PARTNER_QUERY_PARAM,
};
pub use config::{ActivitiesConfig, ActivityRow, ModuleConfig, TiqetsConfig, ViatorConfig};
pub use email_context::{email_context, EmailContextArgs, EmailContextResponse};
pub use guest::{render_explore_detail, render_home_card, render_upcoming_card};
pub use host::render_host_main;
pub use map_markers::{map_markers, MapMarkersResponse, MAX_MARKERS};
pub use tiqets::{FRESH_SECS, MAX_PRODUCTS, STALE_MAX_SECS};
pub use viator::{
    FRESH_SECS as VIATOR_FRESH_SECS, MAX_PRODUCTS as VIATOR_MAX_PRODUCTS,
    STALE_MAX_SECS as VIATOR_STALE_MAX_SECS,
};

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
