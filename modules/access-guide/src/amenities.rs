//! `amenities.list` — ce que le module ajoute aux équipements du logement : la serrure connectée
//! ou la boîte à clés, selon la méthode d'accès. Seulement l'id du catalogue : ni code, ni
//! emplacement.

use portaki_sdk::contracts::amenities::{self, AmenitiesList};
use portaki_sdk::prelude::*;

use crate::config::{HostConfig, PrimaryMethod};

#[portaki_sdk::query(name = "amenities.list", example(label = "Équipements fournis"))]
pub fn amenities_list(ctx: Context) -> Result<AmenitiesList> {
    let id = match HostConfig::load(&ctx)?.method() {
        Some(PrimaryMethod::SmartLock) => Some("smart-lock"),
        Some(PrimaryMethod::Keybox) => Some("key-box"),
        _ => None,
    };
    Ok(amenities::list(id.map(amenities::amenity)))
}
