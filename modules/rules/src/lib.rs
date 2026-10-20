//! Règlement intérieur : une liste de règles que l'hôte ordonne, montrée dans le livret.
//!
//! Le SDK stocke, fusionne, importe et contrôle la collection (`#[collection]`) ; le module dit
//! ce qu'est une règle, ses thèmes, et comment le livret la lit.

pub mod commands;
mod domain;
pub mod email;
mod guest;
mod host;

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::prelude::*;

pub use commands::apply_template::apply_template;
pub use email::vars::email_vars;
pub use guest::explore_detail::render_explore_detail;
pub use guest::home_card::render_home_card;
pub use host::settings::render_host_main;

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

#[portaki_sdk::collection(name = "rules", item = Rule, meta = Display, max = 30, required, import, read, legacy = commands::earlier::earlier_rules)]
pub struct Rules;
