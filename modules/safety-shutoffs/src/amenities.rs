//! `amenities.list` — ce que le module ajoute aux équipements du logement : l'extincteur et le
//! détecteur de fumée, quand un organe affiché est de ce type. Seulement l'id du catalogue : ni
//! titre, ni emplacement, ni consigne.

use portaki_sdk::contracts::amenities::{self, AmenitiesList};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

/// Le type d'organe et l'équipement du catalogue qu'il vaut.
const PROVIDED: [(&str, &str); 2] = [
    ("extinguisher", "fire-extinguisher"),
    ("smoke_detector", "smoke-detector"),
];

/// Seulement les organes que le voyageur voit ([`ModuleConfig::parse_shutoffs`]) : une ligne sans
/// emplacement est un brouillon, pas un équipement.
#[portaki_sdk::query(name = "amenities.list", example(label = "Équipements fournis"))]
pub fn amenities_list(ctx: Context) -> Result<AmenitiesList> {
    let shutoffs = ModuleConfig::load(&ctx)?.parse_shutoffs();
    Ok(amenities::list(
        PROVIDED
            .into_iter()
            .filter(|(kind, _)| shutoffs.iter().any(|row| row.kind_key() == *kind))
            .map(|(_, id)| amenities::amenity(id)),
    ))
}
