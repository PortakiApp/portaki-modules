//! Côté hôte : l'éditeur des listes, la fiche d'un séjour, les statistiques.

mod settings;
mod stats;
mod stay;

pub use settings::render_host_main;
pub use stats::{render_stats_checklist, render_stats_cleaning, stats_summary};
pub use stay::render_host_stay;
