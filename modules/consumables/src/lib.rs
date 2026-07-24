//! Portaki consumables module — host catalog + guest shortage reports.

mod commands;
mod email_send;
mod entities;
mod guest;
mod host;
mod ids;
mod labels;
mod level;
mod queries;
mod status;
mod storage;

pub use commands::{
    replace_items, seed_defaults, submit, update_config, update_status, ConsumableItemInput,
    ReplaceItemsArgs, SubmitArgs, UpdateConfigArgs, UpdateStatusArgs,
};
pub use entities::{ConsumableItem, ConsumableReport};
pub use guest::render_home_card;
pub use host::{render_host_main, render_host_stats, render_host_stay};
pub use level::DEFAULT as LEVEL_DEFAULT;
pub use queries::{
    list_for_stay, list_items, list_open_count, list_recent, ConsumableItemDto, ConsumableReportRow,
    ListForStayArgs, OpenCountDto,
};
pub use status::DEFAULT as STATUS_DEFAULT;
pub use storage::reset_test_store;

portaki_sdk::portaki_module!(
    id = "consumables",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Portaki",
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
