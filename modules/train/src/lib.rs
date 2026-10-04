//! Portaki train module — les prochains trains de la gare du logement.
//!
//! L'hôte donne le nom de sa gare ; le module le résout en `stop_area` Navitia et lit les départs
//! réels par le connecteur [`sncf`]. Les destinations proposées au voyageur sortent du tableau
//! lui-même — aucune liste de gares n'est écrite ici.

mod board;
mod config;
mod guest;
mod host;
mod sncf;

pub use guest::{
    render_explore_detail, render_explore_item, render_home_card, render_upcoming_card,
};
pub use host::render_host_main;

portaki_sdk::portaki_module!(
    id = "train",
    display_name_key = "module.displayName",
    description_key = "module.catalogDescription",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = ModuleType::Official,
    icon = IconName::Train,
    maturity = Maturity::Beta,
    sort_order = 200,
);
