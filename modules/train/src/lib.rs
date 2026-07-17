//! Portaki train module — nearest station schedule glance for guests.
//!
//! v0.1: station info and destination schedules are static Rust constants
//! ([`content`]). No host editor, no storage — see `README.md`.

mod content;
mod guest;

pub use content::{DEFAULT_DESTINATION, DESTINATIONS};
pub use guest::{render_explore_detail, render_home_card};

portaki_sdk::portaki_module!(
    id = "train",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Syntax Labs",
);
