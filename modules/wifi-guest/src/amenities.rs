//! `amenities.list` — ce que le module ajoute aux équipements du logement : le wifi, dès qu'un
//! réseau est configuré. Seulement l'id du catalogue : ni nom de réseau, ni mot de passe.

use portaki_sdk::contracts::amenities::{self, AmenitiesList};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

#[portaki_sdk::query(name = "amenities.list", example(label = "Équipements fournis"))]
pub fn amenities_list(ctx: Context) -> Result<AmenitiesList> {
    let config = ModuleConfig::load(&ctx)?;
    Ok(amenities::list(
        (!config.is_empty()).then(|| amenities::amenity("wifi")),
    ))
}
