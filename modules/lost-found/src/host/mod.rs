//! Host dashboard surfaces — config editor + stay list + stay-action create.
//!
//! Create + status UI lives here (Wasm SDUI). Host shells only embed surfaces.

mod create;
mod main;
mod status_ui;
mod stay;

pub use create::render_host_create;
pub use main::render_host_main;
pub use stay::render_host_stay;
