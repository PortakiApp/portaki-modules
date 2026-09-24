//! Portaki checklist module — guest lists ticked in the booklet, host lists as dated tasks.

mod commands;
mod email_context;
mod entities;
mod guest;
mod host;
mod i18n;
mod ids;
mod labels;
mod lists;
mod queries;
mod show_when;
mod storage;
mod tasks;

pub use commands::{
    complete_item, create_checklist, delete_checklist, uncomplete_item, update_config,
    CreateChecklistArgs, DeleteChecklistArgs, ItemIdArgs, UpdateConfigArgs,
};
pub use email_context::{email_context, EmailContextArgs, EmailContextResponse};
pub use entities::{Checklist, ChecklistCompletion, ChecklistItem, TaskItemState};
pub use guest::{render_home_card, render_post_stay_card};
pub use host::{render_host_main, render_stats_checklist, render_stats_cleaning, stats_summary};
pub use queries::{list_completions, list_items, ChecklistItemDto};
pub use storage::{items_of, list_checklists, reset_test_store};
pub use tasks::{task_complete, task_id, task_toggle, timeline_tasks};

portaki_sdk::portaki_module!(
    id = "checklist",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Portaki",
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
