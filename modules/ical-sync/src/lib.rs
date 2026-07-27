//! Portaki ical-sync module — host calendar feed import (iCal / Airbnb).

mod channel;
mod commands;
mod config;
mod email_i18n;
mod email_send;
mod host;
mod ics;
mod ids;
mod queries;
mod sync_state;

pub use channel::{detect as detect_channel, DetectedChannel, FeedChannelSignals};
pub use commands::{update_config, CalendarInput, UpdateConfigArgs};
pub use config::{load_config, CalendarFeed, CalendarFormat, ModuleConfig, CALENDAR_SLOTS};
pub use host::{render_host_main, render_host_stats};
pub use ics::{parse_stay_rows, FeedParseContext, StayImportRow};
pub use queries::{
    apply_feeds, get_config, list_sources, ApplyFeedsArgs, ApplyFeedsResponse, FeedBody,
    FeedSource, ListSourcesResponse,
};

portaki_sdk::portaki_module!(
    id = "ical-sync",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Portaki",
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";

#[portaki_sdk::capability(required, id = "core.ical.import")]
pub const ICAL_IMPORT: &str = "core.ical.import";
