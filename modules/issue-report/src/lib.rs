//! Portaki issue-report module — guest stay-scoped problem reports.

mod category;
mod commands;
mod config;
mod email_i18n;
mod email_text;
mod entities;
mod guest;
mod host;
mod i18n;
mod ids;
mod queries;
mod status;
mod storage;
mod tasks;

pub use category::Category;
pub use commands::{
    add, reply, resolve, set_status, submit, AddArgs, ReplyArgs, ResolveArgs, SetStatusArgs,
    SubmitArgs,
};
pub use config::ModuleConfig;
pub use email_text::GUEST_TEXT_EMAIL_MAX_CHARS;
pub use entities::{IssueReport, IssueReportEvent};
pub use guest::{render_guest_form, render_home_card};
pub use host::{
    render_host_add, render_host_main, render_host_reports, render_host_stats, render_host_stay,
    stats_summary,
};
pub use queries::{list_for_stay, list_recent, IssueReportRow};
pub use storage::reset_test_store;
pub use tasks::{task_complete, task_toggle, timeline_tasks};

portaki_sdk::portaki_module!(
    id = "issue-report",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::DangerTriangle,
    maturity = Maturity::Stable,
    sort_order = 90,
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";

/// Ce module sait recevoir le signalement d'un souci : un autre module qui en dépend
/// (`#[capability(optional, id = "report.issue")]`) le trouve sur le logement et ouvre ce
/// formulaire, déjà rempli par `category` et `summary` (voir `guest::form`). Aucune commande
/// prêtée : l'envoi reste celui du voyageur, depuis le formulaire de ce module.
#[portaki_sdk::capability(provided, id = "report.issue", path = "issue-report/form")]
pub const REPORT_ISSUE: &str = "report.issue";
