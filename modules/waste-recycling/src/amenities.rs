//! `amenities.list` — ce que le module ajoute aux équipements du logement : le composteur, et le
//! tri sélectif dès qu'il y a une collecte ou un point d'apport. Seulement l'id du catalogue : ni
//! emplacement, ni adresse, ni code du local.

use portaki_sdk::contracts::amenities::{self, AmenitiesList};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

#[portaki_sdk::query(name = "amenities.list", example(label = "Équipements fournis"))]
pub fn amenities_list(ctx: Context) -> Result<AmenitiesList> {
    let config = ModuleConfig::load(&ctx)?;
    let collection = config.has_collection() && !config.collection_days().is_empty();
    let dropoff = config
        .parse_dropoff_points()
        .iter()
        .any(|point| !point.accepted_keys().is_empty());
    Ok(amenities::list(
        [
            (config.has_compost(), "compost"),
            (collection || dropoff, "recycling"),
        ]
        .into_iter()
        .filter(|(provided, _)| *provided)
        .map(|(_, id)| amenities::amenity(id)),
    ))
}
