//! Sections éditoriales : le mot de l'hôte, en blocs de texte mis en forme, dans le livret.
//!
//! Le SDK stocke, fusionne, importe et dessine l'éditeur (`#[collection]`) ; le module dit ce
//! qu'est une section et comment le livret montre le mot.

mod guest;
mod host;

use portaki_sdk::contracts::i18n::I18nText;

pub use guest::{render_explore_sheet, render_home_card};
pub use host::render_host_main;

portaki_sdk::portaki_module!(
    id = "sections",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::Home,
    maturity = Maturity::Stable,
    sort_order = 40,
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";

/// Un bloc du mot : un titre, un texte mis en forme.
#[portaki_sdk::item(key = "title")]
pub struct Section {
    #[field(label = "host.title.label")]
    pub title: I18nText,
    #[field(kind = "rich_text", label = "host.body.label")]
    pub body: I18nText,
}

#[portaki_sdk::collection(name = "sections", item = Section, import)]
pub struct Sections;
