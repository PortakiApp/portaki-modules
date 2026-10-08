//! Portaki pre-arrival form module — ETA, occasion, allergies, and host-configurable questions.

mod commands;
mod config;
mod email_i18n;
mod email_send;
mod entities;
mod guest;
mod host;
mod i18n;
mod ids;
mod queries;
mod show_when;
mod slots;
mod storage;

pub use commands::{send_form_available, submit, SubmitArgs};
pub use config::{ModuleConfig, ShowWhen};
pub use entities::PreArrivalResponse;
pub use guest::{render_guest_form, render_home_card};
pub use host::{render_host_main, render_host_stay};
pub use queries::{get_status, publish_readiness, PreArrivalStatus};
pub use storage::reset_test_store;

portaki_sdk::portaki_module!(
    id = "pre-arrival-form",
    display_name_key = "module.catalogName",
    description_key = "module.description",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::Clipboard,
    maturity = Maturity::Stable,
    sort_order = 50,
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";

/// Les moyens d'arriver que le formulaire propose (§2.20).
///
/// Une liste figée : l'hôte prépare une place de parking, un horaire de train ou un transfert
/// d'aéroport, et « en voiture de location » ne lui dit pas laquelle des trois.
pub const TRANSPORTS: [&str; 4] = ["car", "train", "plane", "other"];

/// Les occasions que le formulaire propose.
///
/// En liste plutôt qu'en texte libre : l'hôte qui lit « anniversaire » sait quoi faire, celui qui
/// lit « c'est spécial pour nous » ne sait pas.
pub const OCCASIONS: [&str; 5] = ["birthday", "honeymoon", "family", "remote_work", "none"];
