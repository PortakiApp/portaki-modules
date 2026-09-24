//! Portaki waste-recycling module — bins and collection schedule.

mod commands;
mod config;
mod guest;
mod host;
mod ids;
mod localized;
mod queries;

pub use commands::{update_config, BinInput, UpdateConfigArgs};
pub use config::{load_config, ModuleConfig};
pub use guest::{render_explore_detail, render_home_card};
pub use host::render_host_main;
pub use queries::get_config;

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
