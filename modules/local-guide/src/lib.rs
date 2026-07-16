//! Portaki local-guide module — nearby spots and host picks.

mod commands;
mod config;
mod guest;
mod queries;
mod render_host;

pub use commands::{update_config, SpotInput, UpdateConfigArgs};
pub use config::{load_config, ModuleConfig};
pub use guest::{render_explore_detail, render_home_card};
pub use queries::get_config;
pub use render_host::render_host_main;

portaki_sdk::portaki_module!(
    id = "local-guide",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Syntax Labs",
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
