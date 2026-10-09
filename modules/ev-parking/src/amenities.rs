//! `amenities.list` — ce que le module ajoute aux équipements du logement : la borne de recharge,
//! dès qu'une place est renseignée. La prise en précision quand l'hôte l'a choisie et qu'elle se
//! dit pareil dans toutes les langues (« Type 2 », « CCS ») ; jamais la place, un code ni le prix.

use portaki_sdk::contracts::amenities::{self, AmenitiesList};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

#[portaki_sdk::query(name = "amenities.list", example(label = "Équipements fournis"))]
pub fn amenities_list(ctx: Context) -> Result<AmenitiesList> {
    let config = ModuleConfig::load(&ctx)?;
    if config.spot_label.is_blank() {
        return Ok(amenities::list([]));
    }
    let charger = amenities::amenity("ev-charger");
    // Vide se lit Type 2 dans le livret, mais l'hôte ne l'a pas dit : pas de précision.
    Ok(amenities::list([match config.charger_type.trim() {
        "type2" => charger.detail("Type 2"),
        "ccs" => charger.detail("CCS"),
        _ => charger,
    }]))
}
