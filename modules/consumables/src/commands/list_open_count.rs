//! `listOpenCount` — le compte des demandes ouvertes.

use portaki_sdk::prelude::*;

use crate::{storage, Products};

/// Open report count for stats cards.
#[portaki_sdk::wire]
#[derive(PartialEq, Eq)]
pub struct OpenCountDto {
    pub open_count: u32,
    pub catalog_count: u32,
}

#[portaki_sdk::query(name = "listOpenCount", example(label = "Signalements ouverts"))]
pub fn list_open_count(ctx: Context) -> Result<OpenCountDto> {
    Ok(OpenCountDto {
        open_count: storage::count_open()? as u32,
        catalog_count: Products::load(&ctx)?.len() as u32,
    })
}
