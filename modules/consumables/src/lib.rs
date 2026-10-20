//! Consommables : le catalogue des produits de base que l'hôte fournit, et les demandes de
//! réassort que le voyageur envoie depuis le livret.
//!
//! Le catalogue et les réglages des demandes sont une collection déclarée : le SDK les stocke,
//! les fusionne, les importe, les contrôle et dessine leur éditeur. Les demandes sont un dépôt
//! déclaré (`#[record]`) : le SDK les stocke avec leur statut et leur historique, et dérive
//! l'envoi, les statuts, la réponse et les tâches de la frise ; le module dit ce qu'est une
//! demande, quand elle est acceptée, et ce que disent les e-mails.

mod commands;
mod config;
mod domain;
mod email;
mod guest;
mod host;

use portaki_sdk::contracts::i18n::I18nText;

use domain::{ids, suggest};

pub use commands::{amenities_list, seed_defaults, stats_summary, SubmitArgs};
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

#[portaki_sdk::collection(name = "products", item = Product, meta = Requests, max = 30, check = config::problems, import, read, legacy = commands::earlier::earlier_products)]
pub struct Products;

/// Où en est une demande : À traiter → Prévu → Livré.
#[portaki_sdk::choice(label = "status")]
pub enum Status {
    #[initial]
    Open,
    Planned,
    #[done]
    Restocked,
}

/// « Il n'y en a plus » ou « Bientôt vide ».
#[portaki_sdk::choice(label = "level")]
pub enum Level {
    Missing,
    Low,
}

/// Une demande : un produit, tel que le voyageur l'a vu, et ce qu'il en dit.
#[portaki_sdk::item(key = "item_label")]
pub struct Request {
    #[field(required, label = "form.item.label")]
    pub item_id: String,
    #[field(required, label = "host.column.product")]
    pub item_label: String,
    #[field(required, label = "form.level.label")]
    pub level: Level,
    #[field(label = "form.note.label")]
    pub note: String,
}

/// Une demande enregistrée, son statut et son historique.
pub type Entry = portaki_sdk::resource::Entry<Request, Status>;

/// Une demande à traiter est une tâche de la frise dès son envoi ; la cocher la passe « Livré ».
#[portaki_sdk::record(name = "requests", item = Request, status = Status, due_hours = 0,
    task_title = "task.title", task_done = "host.main.markRestocked", hooks,
    note_email = "host-reply", legacy = commands::legacy::earlier_requests)]
pub struct Restocks;
