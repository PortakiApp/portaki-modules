//! Portaki checklist module — checkout tasks with stay-scoped completions.

mod commands;
mod email_context;
mod entities;
mod guest;
mod host;
mod ids;
mod labels;
mod queries;
mod storage;

pub use commands::{
    complete_item, replace_items, uncomplete_item, update_config, ChecklistItemInput, ItemIdArgs,
    ReplaceItemsArgs, UpdateConfigArgs,
};
pub use email_context::{email_context, EmailContextArgs, EmailContextResponse};
pub use entities::{ChecklistCompletion, ChecklistItem};
pub use guest::render_home_card;
pub use host::render_host_main;
pub use queries::{list_completions, list_items, ChecklistItemDto};
pub use storage::reset_test_store;

portaki_sdk::portaki_module!(
    id = "checklist",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Portaki",
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
