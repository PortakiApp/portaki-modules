//! Portaki access-guide module — arrival steps, codes, and parking.

mod amenities;
mod commands;
mod config;
mod email_context;
mod email_i18n;
mod guest;
mod host;
mod i18n;
mod queries;
mod reveal;
mod tasks;
mod texts;

pub use amenities::amenities_list;
pub use commands::{on_config_updated, ConfigUpdatedArgs};
pub use config::{
    ArrivalGuide, BuildingAccess, HostConfig, MethodFields, ModuleConfig, ParkingLayer,
    PrimaryMethod, RevealPolicy, StepRow,
};
pub use email_context::{email_context, EmailContextArgs, EmailContextResponse};
pub use guest::{
    render_explore_detail, render_home_card, render_status_cell, render_upcoming_card,
};
pub use host::{render_host_main, render_host_stay};
pub use queries::{map_markers, publish_readiness, MapMarkersResponse};
pub use tasks::{missing_code_tasks, timeline_tasks};
pub use texts::{lang_code, ModuleTexts, StepText};

portaki_sdk::portaki_module!(
    id = "access-guide",
    display_name_key = "module.displayName",
    description_key = "module.catalogDescription",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::Key,
    maturity = Maturity::Stable,
    sort_order = 20,
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";

#[portaki_sdk::capability(provided, id = "amenities.provide")]
pub const AMENITIES: &str = "amenities.provide";
