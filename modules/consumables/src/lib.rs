//! Consommables : le catalogue des produits de base que l'hôte fournit, et les demandes de
//! réassort que le voyageur envoie depuis le livret.
//!
//! Le catalogue et les réglages des demandes sont une collection déclarée : le SDK les stocke,
//! les fusionne, les importe, les contrôle et dessine leur éditeur. Les demandes restent du code
//! du module (entité, envoi, statuts, tâches, e-mails, statistiques).

mod commands;
mod config;
mod domain;
mod email;
mod guest;
mod host;

use portaki_sdk::contracts::i18n::I18nText;

use self::email as email_i18n;
use commands::reports as storage;
use domain::{entities, ids, level, status, suggest};
use email::text as email_text;

pub use commands::reports::reset_test_store;
pub use commands::{
    amenities_list, list_for_stay, list_open_count, list_recent, ConsumableReportRow,
    ListForStayArgs, OpenCountDto,
};
pub use commands::{seed_defaults, submit, update_status, SubmitArgs, UpdateStatusArgs};
pub use commands::{stats_summary, task_complete, task_toggle, timeline_tasks};
pub use domain::entities::ConsumableReport;
pub use domain::level::DEFAULT as LEVEL_DEFAULT;
pub use domain::status::DEFAULT as STATUS_DEFAULT;
pub use email::text::GUEST_TEXT_EMAIL_MAX_CHARS;
pub use guest::{render_guest_form, render_home_card};
pub use host::{render_host_main, render_host_stats, render_host_stay};

portaki_sdk::portaki_module!(
    id = "consumables",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::Package,
    maturity = Maturity::Stable,
    sort_order = 110,
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";

#[portaki_sdk::capability(provided, id = "amenities.provide")]
pub const AMENITIES: &str = "amenities.provide";

/// « Prévenir par » (§2.2) : pas de SMS sur la plateforme.
#[portaki_sdk::choice(label = "host.notify")]
pub enum Channel {
    Email,
    App,
}

/// Un produit du catalogue (§2.1).
#[portaki_sdk::item(key = "label")]
pub struct Product {
    #[field(required, max_chars = 40, label = "host.product.name")]
    pub label: I18nText,
    #[field(label = "host.product.emoji")]
    pub emoji: String,
    #[field(max_chars = 80, label = "host.product.location")]
    pub location: I18nText,
}

/// Les demandes (§2.2) : ouvertes ou non, et, ouvertes, le délai annoncé, les canaux et le plafond
/// par séjour. Rien d'enregistré : ouvertes, par e-mail et notification, 5 par séjour.
#[portaki_sdk::item(meta)]
pub struct Requests {
    #[field(default = true, label = "host.requests.enabled")]
    pub requests_enabled: Option<bool>,
    #[field(
        max_chars = 40,
        visible_when = "requests_enabled=true",
        label = "host.main.restockDelay"
    )]
    pub restock_delay: I18nText,
    #[field(
        kind = "choices",
        default = "[\"email\",\"app\"]",
        visible_when = "requests_enabled=true",
        label = "host.notify"
    )]
    pub notify: Option<Vec<Channel>>,
    #[field(
        between(1, 20),
        default = 5,
        visible_when = "requests_enabled=true",
        label = "host.requests.max"
    )]
    pub max_requests: Option<u32>,
}

#[portaki_sdk::collection(name = "products", item = Product, meta = Requests, max = 30, check = config::problems, import, legacy = commands::earlier::earlier_products)]
pub struct Products;
