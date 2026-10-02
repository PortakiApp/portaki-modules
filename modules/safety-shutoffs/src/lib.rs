//! Portaki safety-shutoffs module — where to cut the power, the water and the gas.

mod config;
mod guest;
mod host;
mod i18n;
mod queries;

pub use config::{ModuleConfig, ShutoffRow, MAX_SHUTOFFS};
pub use guest::{render_explore_detail, render_home_card};
pub use host::render_host_main;
pub use queries::publish_readiness;

portaki_sdk::portaki_module!(
    id = "safety-shutoffs",
    display_name_key = "module.displayName",
    description_key = "module.catalogDescription",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::Shield,
    maturity = Maturity::Stable,
    sort_order = 180,
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
