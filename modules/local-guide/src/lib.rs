//! Portaki local-guide module — nearby spots and host picks.

mod activities;
mod affiliate;
mod commands;
mod config;
mod email_context;
mod guest;
mod host;
mod ids;
mod map_markers;
mod queries;

pub use affiliate::{
    normalize_curated_url, search_url, CuratedUrlError, MAX_CURATED_LINKS, PARTNER_ID,
    PARTNER_QUERY_PARAM,
};
pub use commands::{
    update_config, ActivityInput, SpotInput, UpdateConfigArgs, ERR_ACTIVITIES_TOO_MANY,
    ERR_ACTIVITY_URL_NOT_GYG,
};
pub use config::{load_config, ActivitiesConfig, ActivityRow, ModuleConfig};
pub use email_context::{email_context, EmailContextArgs, EmailContextResponse};
pub use guest::{render_explore_detail, render_home_card, render_upcoming_card};
pub use host::render_host_main;
pub use map_markers::{map_markers, MapMarkersResponse, MAX_MARKERS};
pub use queries::get_config;

portaki_sdk::portaki_module!(
    id = "local-guide",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Portaki",
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
