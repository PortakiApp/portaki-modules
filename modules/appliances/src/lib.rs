//! Appareils : les modes d'emploi des appareils du logement, rangés par pièce dans le livret.
//!
//! Le SDK stocke, fusionne, importe et contrôle la collection (`#[collection]`) et dessine
//! l'éditeur ; le module dit ce qu'est un appareil et comment le livret le montre.

pub mod commands;
mod config;
mod domain;
mod guest;
mod host;

use portaki_sdk::contracts::i18n::I18nText;

pub use commands::stats_summary::stats_summary;
pub use guest::explore_detail::render_explore_detail;
pub use guest::explore_item::render_explore_item;
pub use guest::home_card::render_home_card;
pub use host::appliances_views::render_host_stats;
pub use host::settings::render_host_main;

portaki_sdk::portaki_module!(
    id = "appliances",
    display_name_key = "module.displayName",
    description_key = "module.catalogDescription",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::Plug,
    maturity = Maturity::Stable,
    sort_order = 120,
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";

/// « Un souci avec cet appareil ? » ouvre le module qui sait recevoir un signalement, s'il y en a
/// un sur le logement : pas d'id de module ici, le fournisseur se trouve par cette capacité, et
/// sans lui le bouton disparaît.
#[portaki_sdk::capability(optional, id = "report.issue")]
pub const REPORT_ISSUE: &str = "report.issue";

/// Une étape du mode d'emploi : une consigne, et un schéma si besoin (§2.2).
#[portaki_sdk::item]
pub struct Step {
    #[field(max_chars = 280, label = "host.device.steps.text")]
    pub text: I18nText,
    #[field(kind = "photo", label = "host.device.steps.image")]
    pub image: String,
}

/// Un appareil (§2.2). Le nom, la pièce, les consignes sont traduits ; le reste vaut pour toutes
/// les langues.
#[portaki_sdk::item(key = "name")]
pub struct Device {
    #[field(required, max_chars = 60, label = "host.device.name")]
    pub name: I18nText,
    #[field(label = "host.device.emoji")]
    pub emoji: String,
    #[field(max_chars = 30, label = "host.device.location")]
    pub location: I18nText,
    #[field(max_chars = 60, label = "host.device.model")]
    pub model: String,
    #[field(label = "host.device.featured")]
    pub featured: bool,
    #[field(label = "host.device.status.hidden")]
    pub hidden: bool,
    #[field(max_rows = 15, label = "host.device.steps")]
    pub steps: Vec<Step>,
    #[field(max_chars = 280, label = "host.device.safetyNote")]
    pub safety_note: I18nText,
    #[field(
        kind = "file",
        accept = "application/pdf",
        max_bytes = 20971520,
        label = "host.device.manualFile"
    )]
    pub manual_file: String,
    #[field(kind = "url", label = "host.device.manualUrl")]
    pub manual_url: String,
    #[field(kind = "url", label = "host.device.videoUrl")]
    pub video_url: String,
    #[field(
        kind = "rich_text",
        max_chars = 3000,
        label = "host.device.description"
    )]
    pub description: I18nText,
}

/// La carte d'accueil et ce qui vaut pour tous les appareils (§2.1, §3.1).
#[portaki_sdk::item(meta)]
pub struct Display {
    #[field(between(2, 6), label = "host.featured.label")]
    pub featured_limit: u32,
    #[field(label = "host.roomOrder.label")]
    pub room_order: Vec<String>,
    #[field(max_chars = 120, label = "host.paperManuals.label")]
    pub paper_manuals_location: I18nText,
}

#[portaki_sdk::collection(name = "devices", item = Device, meta = Display, max = 60, check = config::problems, import, read, legacy = commands::earlier::earlier_devices)]
pub struct Devices;
