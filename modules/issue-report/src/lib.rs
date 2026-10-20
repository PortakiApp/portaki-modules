//! Signalements : un voyageur signale un problème pendant son séjour, l'hôte le suit.
//!
//! Le SDK stocke les signalements (`#[record]`), leur statut, leur historique, la boîte de
//! réception, le formulaire et la tâche de frise ; le module dit ce qu'est un signalement, quand
//! il est accepté, et ce que disent les e-mails.

pub mod commands;
mod config;
mod email;
mod guest;
mod host;

use portaki_sdk::prelude::*;
use portaki_sdk::resource::{Choice, Entry, RecordHooks};

pub use config::{compact, ModuleConfig, Phase, RESPONSE_TIMES};
pub use guest::form::render_guest_form;
pub use guest::home_card::render_home_card;
pub use host::reports::render_host_reports;
pub use host::settings::render_host_main;

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

#[portaki_sdk::capability(provided, id = "report.issue", path = "issue-report/form")]
pub const REPORT_ISSUE: &str = "report.issue";

#[portaki_sdk::choice(label = "status")]
pub enum Status {
    #[initial]
    New,
    InProgress,
    #[done]
    Resolved,
}

#[portaki_sdk::choice(label = "form.category")]
pub enum Category {
    #[icon(IconName::Plug)]
    Appliance,
    #[icon(IconName::Sparkles)]
    Cleanliness,
    #[icon(IconName::Volume2)]
    Noise,
    #[icon(IconName::Key)]
    Access,
    #[icon(IconName::Wifi)]
    Wifi,
    #[icon(IconName::Sun)]
    Outdoor,
    #[icon(IconName::MessageCircle)]
    Other,
}

/// Ce que le voyageur signale.
#[portaki_sdk::item(key = "summary")]
pub struct Report {
    #[field(required, label = "form.category.label")]
    pub category: Category,
    #[field(required, max_chars = 200, label = "form.summary.label")]
    pub summary: String,
    #[field(max_chars = 2000, label = "form.details.label")]
    pub details: String,
    #[field(kind = "photo", label = "form.photo.label")]
    pub photo: String,
}

#[portaki_sdk::record(name = "reports", item = Report, status = Status, due_hours = 24,
    task_title = "task.title", task_done = "host.detail.resolve", hooks,
    legacy = commands::earlier::earlier_reports)]
pub struct Reports;

impl RecordHooks for Reports {
    type Item = Report;
    type Status = Status;

    /// Une catégorie retirée par l'hôte, ou une période fermée, ne passe pas — même depuis un
    /// formulaire resté ouvert.
    fn guard(ctx: &Context, report: &Report) -> Result<()> {
        let config = ModuleConfig::load(ctx)?;
        if !config.categories().contains(&report.category.wire()) {
            return Err(PortakiError::Host("category_not_offered".into()));
        }
        let stay = ctx.stay.as_ref();
        let now = portaki_sdk::host::time::now()?;
        if !config.open_at(
            now,
            stay.and_then(|s| s.checkin_at),
            stay.and_then(|s| s.checkout_at),
        ) {
            return Err(PortakiError::Host("out_of_phase".into()));
        }
        Ok(())
    }

    fn on_created(_: &Context, entry: &Entry<Report, Status>) -> Result<()> {
        email::to_host::to_host(entry)
    }

    fn on_note(_: &Context, entry: &Entry<Report, Status>, text: &str) -> Result<()> {
        email::to_guest::to_guest(entry, text)
    }
}
