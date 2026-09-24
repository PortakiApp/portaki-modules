//! Host dashboard surfaces — config editor, stats list, stay list + stay-action create.
//!
//! Create + status UI lives here (Wasm SDUI). Host shells only embed surfaces.

mod create;
mod stats;
mod status_ui;
mod stay;

pub use create::render_host_create;
pub use stats::{render_host_stats, stats_summary};
pub use stay::render_host_stay;
