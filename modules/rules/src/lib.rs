//! Règlement intérieur : une liste de règles que l'hôte ordonne, montrée dans le livret.
//!
//! Le SDK stocke, fusionne, importe et contrôle la collection (`#[collection]`) ; le module dit
//! ce qu'est une règle, ses thèmes, et comment le livret la lit.

pub mod email;
mod guest;
mod host;

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::prelude::*;

pub use email::email_vars;
pub use guest::{render_explore_detail, render_home_card};
pub use host::{apply_template, render_host_main};

portaki_sdk::portaki_module!(
    id = "rules",
    display_name_key = "module.displayName",
    description_key = "module.catalogDescription",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::Scale,
    maturity = Maturity::Stable,
    sort_order = 30,
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";

#[portaki_sdk::choice(label = "rule.status")]
pub enum RuleStatus {
    #[icon(IconName::Info)]
    Neutral,
    #[icon(IconName::TriangleAlert)]
    Important,
    #[icon(IconName::Check)]
    Allowed,
    #[icon(IconName::X)]
    Forbidden,
}

#[portaki_sdk::choice(label = "rule.theme")]
pub enum Theme {
    Other,
    Noise,
    Pets,
    Smoking,
    Parties,
    Visitors,
    Cleaning,
    Safety,
}

/// Une règle du logement.
#[portaki_sdk::item(key = "title")]
pub struct Rule {
    #[field(required, max_chars = 60, label = "host.rule.title")]
    pub title: I18nText,
    #[field(max_chars = 280, label = "host.rule.detail")]
    pub detail: I18nText,
    #[field(label = "host.rule.status")]
    pub status: RuleStatus,
    #[field(label = "host.rule.theme")]
    pub theme: Theme,
    #[field(kind = "time_range", label = "host.rule.hours")]
    pub hours: String,
}

/// Combien de règles la carte d'accueil montre.
#[portaki_sdk::item(meta)]
pub struct Display {
    #[field(between(2, 6), label = "host.cardLimit.label")]
    pub card_limit: u32,
}

#[portaki_sdk::collection(name = "rules", item = Rule, meta = Display, max = 30, required, import)]
pub struct Rules;

impl Display {
    /// Les règles de la carte : 4 tant que l'hôte n'a rien choisi.
    pub fn limit(&self) -> usize {
        if self.card_limit == 0 {
            4
        } else {
            self.card_limit as usize
        }
    }
}

impl Rule {
    /// « 22:00 – 08:00 », vide sans horaires.
    pub fn hours_label(&self) -> String {
        portaki_sdk::sdui::hours::parse_range(&self.hours)
            .map(|(start, end)| format!("{start} – {end}"))
            .unwrap_or_default()
    }

    /// L'icône du livret, d'après le thème.
    pub fn icon(&self) -> &'static str {
        match self.theme {
            Theme::Noise => "clock-circle",
            Theme::Pets => "gift",
            Theme::Smoking => "x",
            Theme::Parties | Theme::Visitors => "users",
            _ => "check-circle",
        }
    }

    /// Le thème que le texte évoque, pour le suggérer à l'hôte (§4).
    pub fn suggested_theme(&self) -> Option<Theme> {
        let text = format!("{} {}", self.title.fr, self.detail.fr).to_lowercase()
            + &format!(" {} {}", self.title.en, self.detail.en).to_lowercase();
        THEME_WORDS
            .iter()
            .find(|(_, words)| words.iter().any(|word| text.contains(word)))
            .map(|(theme, _)| *theme)
    }
}

#[rustfmt::skip] // une table de mots, une ligne par thème
const THEME_WORDS: [(Theme, &[&str]); 7] = [
    (Theme::Noise, &["silence", "bruit", "calme", "musique", "noise", "quiet", "music"]),
    (Theme::Pets, &["animal", "animaux", "chien", "chat", "pet", "dog", "cat"]),
    (Theme::Smoking, &["fum", "tabac", "cigarette", "smok", "vap"]),
    (Theme::Parties, &["fête", "fete", "soirée", "party", "parties", "événement", "event"]),
    (Theme::Visitors, &["visiteur", "invité", "visitor", "guest"]),
    (Theme::Cleaning, &["ménage", "poubelle", "déchet", "vaisselle", "clean", "trash", "rubbish", "dishes"]),
    (Theme::Safety, &["sécurité", "feu", "bougie", "piscine", "safety", "fire", "candle", "pool"]),
];
