//! Portaki rules module — structured bilingual house rules.

mod commands;
mod content;
mod email_context;
mod entities;
mod guest;
mod host;
mod ids;
mod queries;
mod store;

pub use commands::{save_content, update_config, RuleItemInput, SaveContentArgs};
pub use content::{RuleItem, RulesPayload};
pub use email_context::{email_context, EmailContextArgs, EmailContextResponse};
pub use entities::RulesContent;
pub use guest::{render_explore_detail, render_home_card};
pub use host::render_host_main;
pub use queries::{get_content, GetContentArgs, RulesContentView};
pub use store::reset_test_store;

portaki_sdk::portaki_module!(
    id = "rules",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Portaki",
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
