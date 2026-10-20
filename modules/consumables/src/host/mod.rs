//! Côté hôte : la page des réglages et des demandes reçues, la fiche d'un séjour, les statistiques.

mod charts;
mod report_ui;
mod settings;
mod stats;
mod stay;
mod stock;

pub use settings::render_host_main;
pub use stay::render_host_stay;
pub use stock::render_host_stats;
