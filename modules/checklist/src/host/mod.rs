//! Côté hôte : l'éditeur des listes, la fiche d'un séjour, les statistiques — une surface par
//! fichier.

mod checklist;
mod cleaning;
mod settings;
pub(crate) mod stats;
mod stay;

pub use checklist::render_stats_checklist;
pub use cleaning::render_stats_cleaning;
pub use settings::render_host_main;
pub use stay::render_host_stay;
