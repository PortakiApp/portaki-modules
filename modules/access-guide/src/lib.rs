//! Portaki access-guide module — arrival steps, codes, and parking.

mod commands;
mod config;
mod email_context;
mod guest;
mod queries;
mod render_host;
mod reveal;
mod texts;

pub use commands::{update_config, StepInput, UpdateConfigArgs};
pub use config::{
    load_config, ArrivalGuide, BuildingAccess, MethodFields, ModuleConfig, ParkingLayer,
    PrimaryMethod, RevealPolicy,
};
pub use email_context::{email_context, EmailContextArgs, EmailContextResponse};
pub use guest::{render_explore_detail, render_home_card};
pub use queries::get_config;
pub use render_host::render_host_main;
pub use texts::{lang_code, load_texts, ModuleTexts, StepText};

portaki_sdk::portaki_module!(
    id = "access-guide",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Syntax Labs",
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
