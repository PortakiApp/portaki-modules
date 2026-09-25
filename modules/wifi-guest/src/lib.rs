//! Portaki wifi-guest module — guest Wi-Fi SSID and password for booklets.

mod config;
mod email_context;
mod guest;
mod host;
mod reveal;

pub use config::{ModuleConfig, RevealPolicy};
pub use email_context::{email_context, EmailContextArgs, EmailContextResponse};
pub use guest::{render_explore_detail, render_home_card};
pub use host::render_host_main;

portaki_sdk::portaki_module!(
    id = "wifi-guest",
    display_name_key = "module.displayName",
    description_key = "module.catalogDescription",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::Wifi,
    maturity = Maturity::Stable,
    sort_order = 10,
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
