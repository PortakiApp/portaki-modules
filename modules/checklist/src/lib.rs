//! Checklists : des listes que le voyageur coche dans le livret, et des listes d'équipe qui
//! deviennent des tâches datées autour de chaque séjour.
//!
//! Les listes et leurs étapes sont une collection déclarée : le SDK les stocke, les fusionne, les
//! importe, les contrôle et dessine leur éditeur. Restent du code du module : les coches du
//! voyageur, les tâches de l'équipe et leur état, le rappel, les statistiques.

mod commands;
mod config;
mod domain;
mod email;
mod guest;
mod host;

use portaki_sdk::contracts::i18n::I18nText;

use self::email as email_i18n;
use commands::{entities, storage};
use domain::{ids, lists, show_when};

pub use commands::complete_item::{complete_item, ItemIdArgs};
pub use commands::create_checklist::{create_checklist, CreateChecklistArgs};
pub use commands::email_context::{email_context, EmailContextArgs, EmailContextResponse};
pub use commands::entities::{ChecklistCompletion, TaskItemState};
pub use commands::list_completions::list_completions;
pub use commands::publish_readiness::publish_readiness;
pub use commands::send_departure_reminder::send_departure_reminder;
pub use commands::set_completed::{set_completed, SetCompletedArgs};
pub use commands::stats_summary::stats_summary;
pub use commands::storage::reset_test_store;
pub use commands::task::task_id;
pub use commands::task_complete::task_complete;
pub use commands::task_toggle::task_toggle;
pub use commands::timeline_tasks::timeline_tasks;
pub use commands::uncomplete_item::uncomplete_item;
pub use domain::{all_lists, booklet_lists};
pub use guest::{render_home_card, render_post_stay_card};
pub use host::{render_host_main, render_host_stay, render_stats_checklist, render_stats_cleaning};

portaki_sdk::portaki_module!(
    id = "checklist",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::CheckCircle,
    maturity = Maturity::Stable,
    sort_order = 60,
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";

/// Qui coche la liste : le voyageur dans son livret, ou l'équipe dans une tâche.
#[portaki_sdk::choice(label = "host.audience")]
pub enum Audience {
    Guest,
    Host,
}

/// « Afficher » (§2.1) : quand la liste s'ouvre dans le livret.
#[portaki_sdk::choice(label = "host.trigger")]
pub enum GuestTrigger {
    #[serde(rename = "duringStay")]
    DuringStay,
    #[serde(rename = "atDeparture")]
    AtDeparture,
    #[serde(rename = "departureDay")]
    DepartureDay,
    #[serde(rename = "beforeArrival")]
    BeforeArrival,
}

/// « Créer la tâche » (§2.2) : quels séjours reçoivent une tâche.
#[portaki_sdk::choice(label = "host.trigger")]
pub enum HostTrigger {
    #[serde(rename = "afterEachDeparture")]
    AfterEachDeparture,
    #[serde(rename = "beforeEachArrival")]
    BeforeEachArrival,
    #[serde(rename = "atDepartureBeforeCleaning")]
    AtDepartureBeforeCleaning,
    #[serde(rename = "onlyIfNextArrival")]
    OnlyIfNextArrival,
}

/// « À terminer avant » (§2.2).
#[portaki_sdk::choice(label = "host.deadline")]
pub enum Deadline {
    #[serde(rename = "nextArrivalMinus2h")]
    NextArrivalMinus2h,
    #[serde(rename = "nextArrival")]
    NextArrival,
    #[serde(rename = "departureEvening")]
    DepartureEvening,
    #[serde(rename = "arrival")]
    Arrival,
}

/// Une étape (§2.9) : son texte, sa précision, sa rubrique ; une photo pour la cocher (équipe).
#[portaki_sdk::item]
pub struct Step {
    #[field(required, max_chars = 80, label = "host.step.label")]
    pub label: I18nText,
    #[field(max_chars = 120, label = "host.step.detail")]
    pub detail: I18nText,
    #[field(max_chars = 30, label = "host.step.zone")]
    pub zone: I18nText,
    #[field(label = "host.step.photo")]
    pub photo: bool,
}

/// Une checklist. Ses réglages dépendent de qui la coche : le livret (§2.1) ou l'équipe (§2.2).
#[portaki_sdk::item(key = "name")]
pub struct List {
    #[field(required, label = "host.field.name")]
    pub name: I18nText,
    #[field(label = "host.field.audience")]
    pub audience: Audience,
    /// L'icône du modèle de départ.
    pub icon: String,
    #[field(visible_when = "audience=guest", label = "host.field.guestTrigger")]
    pub guest_trigger: GuestTrigger,
    #[field(
        between(3, 10),
        default = 5,
        visible_when = "audience=guest",
        label = "host.field.visibleLimit"
    )]
    pub visible_limit: Option<u32>,
    #[field(
        max_chars = 120,
        visible_when = "audience=guest",
        label = "host.field.doneMessage"
    )]
    pub done_message: I18nText,
    #[field(
        default = true,
        visible_when = "audience=guest",
        label = "host.toggle.remind"
    )]
    pub remind: Option<bool>,
    #[field(visible_when = "audience=host", label = "host.field.hostTrigger")]
    pub host_trigger: HostTrigger,
    #[field(visible_when = "audience=host", label = "host.field.assignee")]
    pub assignee: String,
    #[field(visible_when = "audience=host", label = "host.field.deadline")]
    pub deadline: Deadline,
    #[field(visible_when = "audience=host", label = "host.toggle.notifyAssignee")]
    pub notify_assignee: bool,
    #[field(visible_when = "audience=host", label = "host.toggle.alertHost")]
    pub alert_host: bool,
    #[field(max_rows = 40, label = "host.steps.title")]
    pub steps: Vec<Step>,
}

#[portaki_sdk::collection(name = "lists", item = List, check = config::problems, readiness = false, import, legacy = commands::earlier::earlier_lists)]
pub struct Lists;
