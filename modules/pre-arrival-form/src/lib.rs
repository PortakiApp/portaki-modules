//! Portaki pre-arrival form module — ETA, occasion, allergies, and host-configurable questions.

mod commands;
mod config;
mod entities;
mod guest;
mod host;
mod ids;
mod queries;
mod show_when;
mod storage;

pub use commands::{submit, update_config, SubmitArgs, UpdateConfigArgs};
pub use config::{load_config, FormQuestions, ModuleConfig, ShowWhen};
pub use entities::PreArrivalResponse;
pub use guest::render_home_card;
pub use host::{render_host_main, render_host_stay};
pub use queries::{get_status, PreArrivalStatus};
pub use storage::reset_test_store;

portaki_sdk::portaki_module!(
    id = "pre-arrival-form",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Portaki",
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
