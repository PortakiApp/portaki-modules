//! Portaki guest-reviews module — post-stay thank-you and review CTAs.

mod commands;
mod config;
mod guest;
mod host;
mod ids;
mod localized;
mod queries;

pub use commands::{submit_review, update_config, SubmitReviewArgs, UpdateConfigArgs};
pub use config::{load_config, ModuleConfig};
pub use guest::render_home_card;
pub use host::render_host_main;
pub use queries::get_config;

portaki_sdk::portaki_module!(
    id = "guest-reviews",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Portaki",
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
