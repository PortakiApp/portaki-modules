//! Host surfaces — the config editor and the « Avis » statistics.

mod main;
mod stats;
mod stay;

pub(crate) use main::public_reviews_problem;
pub use main::render_host_main;
pub(crate) use stats::average;
pub use stats::{render_host_stats, stats_summary};
pub use stay::render_host_stay;
