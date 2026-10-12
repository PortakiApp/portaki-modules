//! Portaki sections module — the host's word of welcome and the booklet's long texts.
//!
//! A core module of the booklet: installed on every property, always active, no switch. The host
//! writes in the listing's « Fiche & coordonnées » card ([`render_host_main`]).

mod config;
mod content;
mod entities;
mod guest;
mod host;
mod i18n;
mod model;
mod queries;
mod store;

pub use config::{ModuleConfig, SectionRow, ShowFrom};
pub use entities::{SectionItem, SectionItemLocale};
pub use guest::{opens_by, render_explore_sheet, render_home_card};
pub use host::render_host_main;
pub use model::SectionView;
pub use queries::{list_sections, publish_readiness, ListSectionsArgs};
pub use store::{reset_test_store, seed_legacy_section};

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
    core,
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
