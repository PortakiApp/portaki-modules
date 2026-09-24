//! Portaki guest-reviews module — post-stay thank-you and review CTAs.

mod commands;
mod config;
mod email_i18n;
mod email_text;
mod guest;
mod host;
mod i18n;
mod ids;
mod localized;
mod queries;

pub use commands::{
    submit_review, update_config, StoredReview, SubmitReviewArgs, UpdateConfigArgs,
};
pub use config::{load_config, ModuleConfig};
pub use email_text::GUEST_TEXT_EMAIL_MAX_CHARS;
pub use guest::{render_home_card, render_post_stay_card};
pub use host::{render_host_main, render_host_stats, stats_summary};
pub use queries::get_config;

portaki_sdk::portaki_module!(
    id = "guest-reviews",
    display_name_key = "module.displayName",
    description_key = "module.catalogDescription",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::Star,
    maturity = Maturity::Stable,
    sort_order = 100,
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
