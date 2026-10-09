//! Portaki waste-recycling module — bins and collection schedule.

mod amenities;
mod collection;
mod config;
mod email_context;
mod guest;
mod host;
mod i18n;
mod queries;

pub use amenities::amenities_list;
pub use collection::{next_collection, Departure, NextCollection};
pub use config::{BinRow, DropoffRow, ModuleConfig};
pub use email_context::email_blocks;
pub use guest::{render_explore_detail, render_home_card};
pub use host::{render_host_main, MAX_BINS};
pub use queries::{map_markers, publish_readiness, MapMarkersResponse, MAX_MARKERS};

portaki_sdk::portaki_module!(
    id = "waste-recycling",
    display_name_key = "module.displayName",
    description_key = "module.catalogDescription",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::Recycle,
    maturity = Maturity::Stable,
    sort_order = 160,
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";

#[portaki_sdk::capability(provided, id = "amenities.provide")]
pub const AMENITIES: &str = "amenities.provide";
