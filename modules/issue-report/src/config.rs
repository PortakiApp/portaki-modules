//! Les réglages du formulaire : catégories, périodes, photo, urgence, réponse — déclarés pour la
//! plateforme (`#[config]`) et pour le formulaire hôte dérivé (`form`).

use portaki_sdk::config::check;
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::prelude::*;
use portaki_sdk::resource::Choice;

use crate::Category;

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
