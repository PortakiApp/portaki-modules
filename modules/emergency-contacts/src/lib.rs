//! Portaki emergency-contacts module — useful numbers and host line.

mod config;
mod email_context;
mod guest;
mod host;
mod ids;
mod localized;

pub use config::{ContactRow, ModuleConfig};
pub use email_context::{email_context, EmailContextArgs, EmailContextResponse};
pub use guest::{render_explore_detail, render_home_card};
pub use host::render_host_main;

portaki_sdk::portaki_module!(
    id = "emergency-contacts",
    display_name_key = "module.displayName",
    description_key = "module.catalogDescription",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::Phone,
    maturity = Maturity::Stable,
    sort_order = 70,
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
