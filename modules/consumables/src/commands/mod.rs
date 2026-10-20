//! Les opérations du module : le catalogue de départ, la tuile de statistiques, les produits pour
//! les autres modules, et ce que le module ajoute à l'envoi d'une demande. Le reste des demandes
//! (statuts, réponse, lectures, tâches) est généré par `#[record]`.

pub mod amenities_list;
pub mod earlier;
pub mod seed_defaults;
pub mod stats_summary;
pub mod submit;

pub use amenities_list::amenities_list;
pub use seed_defaults::{seed_defaults, SEED_DEFAULTS};
pub use stats_summary::stats_summary;
pub use submit::SubmitArgs;
