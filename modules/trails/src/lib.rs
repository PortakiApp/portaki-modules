//! Portaki trails module — the walks around the property, grouped by level.

mod config;
mod format;
pub mod gpx;
mod guest;
mod host;
mod i18n;
mod queries;

pub use config::{ModuleConfig, TrailRow, LEVELS, MAX_TRAILS, SHAPES};
pub use guest::{
    render_explore_detail, render_explore_item, render_home_card, render_property_public,
};
pub use host::render_host_main;
pub use queries::{map_markers, publish_readiness, MapMarkersResponse};

portaki_sdk::portaki_module!(
    id = "trails",
    display_name_key = "module.displayName",
    description_key = "module.catalogDescription",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::Mountain,
    maturity = Maturity::Stable,
    sort_order = 190,
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
