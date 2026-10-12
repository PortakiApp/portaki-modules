//! Host dashboard surfaces — property guidance, stay-scoped responses, resend, stats tile.

mod main;
mod resend;
mod stats;
mod stay;

pub use main::render_host_main;
pub use resend::render_host_resend;
pub use stats::{render_host_stats, stats_summary};
pub use stay::render_host_stay;
