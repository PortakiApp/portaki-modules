//! Portaki ical-sync module — host calendar feed import (iCal / Airbnb).

mod commands;
mod config;
mod host;
mod ics;
mod ids;
mod queries;

pub use commands::{update_config, UpdateConfigArgs};
pub use config::{load_config, ModuleConfig};
pub use host::render_host_main;
pub use ics::{parse_stay_rows, StayImportRow};
pub use queries::{
    apply_feeds, get_config, list_sources, ApplyFeedsArgs, ApplyFeedsResponse, FeedBody,
    FeedSource, ListSourcesResponse,
};

portaki_sdk::portaki_module!(
    id = "ical-sync",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Syntax Labs",
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";

#[portaki_sdk::capability(required, id = "core.ical.import")]
pub const ICAL_IMPORT: &str = "core.ical.import";
