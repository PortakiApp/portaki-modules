//! Signalements : un voyageur signale un problème pendant son séjour, l'hôte le suit.
//!
//! Le SDK stocke les signalements (`#[record]`), leur statut, leur historique, la boîte de
//! réception, le formulaire et la tâche de frise ; le module dit ce qu'est un signalement, quand
//! il est accepté, et ce que disent les e-mails.

mod email;
mod guest;
mod host;

use portaki_sdk::config::check;
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::prelude::*;
use portaki_sdk::resource::{Choice, Entry, RecordHooks};

pub use guest::{render_guest_form, render_home_card};
pub use host::{render_host_main, render_host_reports};

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
    task_title = "task.title", task_done = "host.detail.resolve", hooks)]
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
        email::to_host(entry)
    }

    fn on_note(_: &Context, entry: &Entry<Report, Status>, text: &str) -> Result<()> {
        email::to_guest(entry, text)
    }
}

pub const RESPONSE_TIMES: [&str; 3] = ["within_day", "within_hour", "none"];

#[portaki_sdk::config(form)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    #[field(kind = "structured", choice = Category, choice_labels = "host.category",
        section = "host.form.title", label = "host.category.title")]
    pub categories: Option<Vec<String>>,
    #[field(kind = "structured", choice = Phase, choice_labels = "host.phase",
        section = "host.form.title", label = "host.phase.title")]
    pub phases: Option<Vec<String>>,
    #[field(section = "host.form.title", label = "host.photo.label")]
    pub photo_allowed: Option<bool>,
    #[field(section = "host.form.title", label = "host.urgent.label")]
    pub urgent_phone: Option<bool>,
    #[field(
        local,
        input = "phone",
        visible_when = "urgent_phone=true",
        section = "host.form.title",
        label = "host.urgentNumber.label"
    )]
    pub urgent_number: String,
    #[field(
        input = "textarea",
        section = "host.reception.title",
        label = "host.autoReply.label"
    )]
    pub auto_reply: I18nText,
    #[field(kind = "select", options = ["within_hour", "within_day", "none"], input = "segmented",
        section = "host.reception.title", label = "host.responseTime.label")]
    pub response_time: String,
}

#[portaki_sdk::choice(label = "host.phase")]
pub enum Phase {
    Before,
    During,
    After,
}

impl ModuleConfig {
    /// Les catégories proposées : toutes tant que l'hôte n'a rien choisi.
    pub fn categories(&self) -> Vec<&'static str> {
        let all = Category::ALL.iter().map(|c| c.wire());
        match &self.categories {
            Some(set) => all.filter(|c| set.iter().any(|s| s == c)).collect(),
            None => all.collect(),
        }
    }

    /// Les périodes où le formulaire est proposé : pendant le séjour par défaut.
    pub fn phases(&self) -> Vec<&'static str> {
        let all = Phase::ALL.iter().map(|p| p.wire());
        match &self.phases {
            Some(set) => all.filter(|p| set.iter().any(|s| s == p)).collect(),
            None => vec!["during"],
        }
    }

    pub fn open_at(
        &self,
        now: DateTime<Utc>,
        checkin: Option<DateTime<Utc>>,
        checkout: Option<DateTime<Utc>>,
    ) -> bool {
        let phase = match (checkin, checkout) {
            (Some(checkin), _) if now < checkin => "before",
            (_, Some(checkout)) if now >= checkout => "after",
            _ => "during",
        };
        self.phases().contains(&phase)
    }

    pub fn response_time(&self) -> &'static str {
        RESPONSE_TIMES
            .into_iter()
            .find(|k| *k == self.response_time.trim())
            .unwrap_or(RESPONSE_TIMES[0])
    }

    pub fn problems(&self) -> Vec<(&'static str, I18nText)> {
        let mut problems = Vec::new();
        if self.categories().is_empty() {
            problems.push((
                "categories",
                portaki_sdk::bundle_text!("host.category.required"),
            ));
        }
        if self.phases().is_empty() {
            problems.push(("phases", portaki_sdk::bundle_text!("host.phase.required")));
        }
        if self.urgent_phone.unwrap_or(true) {
            if let Some(error) = check::phone(&compact(&self.urgent_number)) {
                problems.push(("urgent_number", error));
            }
        }
        if let Some(error) = self
            .auto_reply
            .by_language()
            .find_map(|(_, t)| check::max_chars(t, 200))
        {
            problems.push(("auto_reply", error));
        }
        problems
    }

    pub fn error_of(&self, field: &str) -> Option<I18nText> {
        self.problems()
            .into_iter()
            .find(|(name, _)| *name == field)
            .map(|(_, e)| e)
    }
}

pub fn compact(phone: &str) -> String {
    phone
        .chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '.' | '-' | '(' | ')'))
        .collect()
}
