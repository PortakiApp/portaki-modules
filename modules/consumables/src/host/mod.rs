//! Côté hôte : la page des réglages et des demandes reçues, la fiche d'un séjour, les statistiques.

mod report_ui;
mod settings;
mod stats;
mod stay;

pub use settings::render_host_main;
pub use stats::{render_host_stats, stats_summary};
pub use stay::render_host_stay;
