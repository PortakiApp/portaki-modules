//! Host dashboard surfaces — catalog editor + stay list + stats card.

mod main;
mod report_ui;
mod stats;
mod stay;

pub use main::render_host_main;
pub use stats::render_host_stats;
pub use stay::render_host_stay;
