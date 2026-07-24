//! Host dashboard surfaces — config editor + stay declare modal.
//!
//! Create + status UI lives here (Wasm SDUI). Host shells only embed surfaces.

mod create;
mod main;
mod status_ui;
mod stay;

pub use main::render_host_main;
pub use stay::render_host_stay;
