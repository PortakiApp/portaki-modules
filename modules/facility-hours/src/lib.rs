//! Portaki facility-hours module — amenity schedules.

mod config;
mod guest;
mod host;

pub use config::{FacilityRow, ModuleConfig};
pub use guest::{render_explore_detail, render_home_card};
pub use host::render_host_main;

portaki_sdk::portaki_module!(
    id = "facility-hours",
    display_name_key = "module.displayName",
    description_key = "module.catalogDescription",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::ClockCircle,
    maturity = Maturity::Stable,
    sort_order = 150,
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
