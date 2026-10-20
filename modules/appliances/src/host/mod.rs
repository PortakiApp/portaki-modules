//! Côté hôte : la page des réglages et la tuile de statistiques.

mod settings;
mod stats;

pub use settings::render_host_main;
pub use stats::{render_host_stats, stats_summary};
