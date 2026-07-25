//! Portaki issue-report module — guest stay-scoped problem reports.

mod category;
mod commands;
mod entities;
mod guest;
mod host;
mod ids;
mod queries;
mod storage;

pub use commands::{submit, SubmitArgs};
pub use entities::IssueReport;
pub use guest::{render_guest_form, render_home_card};
pub use host::render_host_main;
pub use queries::{list_for_stay, list_recent, IssueReportRow};
pub use storage::reset_test_store;

portaki_sdk::portaki_module!(
    id = "issue-report",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Portaki",
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
