//! Portaki facility-hours module — amenity schedules.

mod commands;
mod config;
mod guest;
mod host;
mod ids;
mod localized;
mod queries;

pub use commands::{update_config, FacilityInput, UpdateConfigArgs};
pub use config::{load_config, ModuleConfig};
pub use guest::{render_explore_detail, render_home_card};
pub use host::render_host_main;
pub use queries::get_config;

portaki_sdk::portaki_module!(
    id = "facility-hours",
    display_name_key = "module.displayName",
    description_key = "module.catalogDescription",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = "official",
    icon = "clock-circle",
    maturity = "stable",
    sort_order = 150,
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
