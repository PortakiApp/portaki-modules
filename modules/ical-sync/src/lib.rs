//! Portaki ical-sync module — host calendar feed import (iCal / Airbnb).

mod channel;
mod commands;
mod config;
mod email_i18n;
mod email_send;
mod host;
mod i18n;
mod ics;
mod ids;
mod queries;
mod sync_state;

pub use channel::{detect as detect_channel, DetectedChannel, FeedChannelSignals};
pub use commands::{update_config, CalendarInput, UpdateConfigArgs};
pub use config::{load_config, CalendarFeed, CalendarFormat, ModuleConfig, CALENDAR_SLOTS};
pub use host::{render_host_main, render_host_stats, stats_summary};
pub use ics::{parse_stay_rows, FeedParseContext, StayImportRow};
pub use queries::{
    apply_feeds, get_config, list_sources, ApplyFeedsArgs, ApplyFeedsResponse, FeedBody,
    FeedSource, ListSourcesResponse,
};

portaki_sdk::portaki_module!(
    id = "ical-sync",
    display_name_key = "module.displayName",
    description_key = "module.catalogDescription",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::Calendar,
    maturity = Maturity::Beta,
    sort_order = 220,
    audience = ModuleAudience::Host,
    scheduled_sync_platform_fetch,
    scheduled_sync_sources = "listSources",
    scheduled_sync_apply = "applyFeeds",
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";

/// La plateforme invoque la synchro ; le plan dit à quelle cadence.
///
/// Remplace `core.ical.import`, retiré du catalogue : il nommait un format là où le cœur ne
/// devrait connaître qu'un service, et promettait une fréquence que l'ordonnanceur ne
/// pouvait pas tenir.
#[portaki_sdk::capability(required, id = "core.modules.scheduled_sync")]
pub const SCHEDULED_SYNC: &str = "core.modules.scheduled_sync";
