//! Host surfaces — the config editor and the « Avis » statistics.

mod main;
mod stats;

pub use main::render_host_main;
pub use stats::{render_host_stats, stats_summary};
