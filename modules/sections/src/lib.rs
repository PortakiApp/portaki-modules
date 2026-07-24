//! Portaki sections module — editorial title + markdown blocks.

mod commands;
mod content;
mod entities;
mod guest;
mod host;
mod ids;
mod model;
mod queries;
mod store;

pub use commands::{
    delete_section, reorder, save_section, DeleteSectionArgs, ReorderArgs, SaveSectionArgs,
};
pub use entities::{SectionItem, SectionItemLocale};
pub use guest::{render_explore_sheet, render_home_card};
pub use host::render_host_main;
pub use model::{SectionLocaleInput, SectionView};
pub use queries::{list_sections, ListSectionsArgs};
pub use store::reset_test_store;

portaki_sdk::portaki_module!(
    id = "sections",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Portaki",
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
