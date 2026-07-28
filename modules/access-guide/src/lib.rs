//! Portaki access-guide module — arrival steps, codes, and parking.

mod commands;
mod config;
mod email_context;
mod email_i18n;
mod guest;
mod host;
mod ids;
mod queries;
mod reveal;
mod texts;

pub use commands::{update_config, StepInput, UpdateConfigArgs};
pub use config::{
    load_config, ArrivalGuide, BuildingAccess, MethodFields, ModuleConfig, ParkingLayer,
    PrimaryMethod, RevealPolicy,
};
pub use email_context::{email_context, EmailContextArgs, EmailContextResponse};
pub use guest::{render_explore_detail, render_home_card, render_upcoming_card};
pub use host::render_host_main;
pub use queries::get_config;
pub use texts::{lang_code, load_texts, ModuleTexts, StepText};

portaki_sdk::portaki_module!(
    id = "access-guide",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Portaki",
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
