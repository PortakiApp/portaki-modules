//! `amenities.list` — ce que le module ajoute aux équipements du logement : les produits de base,
//! dès qu'un produit nommé est au catalogue. Seulement l'id du catalogue : aucun nom de produit.

use portaki_sdk::contracts::amenities::{self, AmenitiesList};
use portaki_sdk::prelude::*;

use crate::labels::labels_from_item;
use crate::storage;

/// Les produits sont des lignes d'entité, pas une config publiée : le catalogue tel que le livret
/// le montre, sans les produits sans nom.
#[portaki_sdk::query(name = "amenities.list", example(label = "Équipements fournis"))]
pub fn amenities_list(_ctx: Context) -> Result<AmenitiesList> {
    let named = storage::list_items()?
        .iter()
        .any(|item| !labels_from_item(item).is_empty());
    Ok(amenities::list(
        named.then(|| amenities::amenity("pantry-basics")),
    ))
}
