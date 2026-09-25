//! Load config for guest surfaces.

use portaki_sdk::prelude::*;

use crate::config::{BinRow, ModuleConfig};

pub struct GuestData {
    pub bins: Vec<BinRow>,
    pub collection_schedule: String,
    pub locale: String,
}

/// The config to show, or `None` when the host has filled in nothing yet.
pub fn load_guest_data(ctx: &GuestContext) -> Result<Option<GuestData>> {
    let config = ModuleConfig::load(ctx)?;
    if config.is_empty() {
        return Ok(None);
    }

    Ok(Some(GuestData {
        bins: config.parse_bins(),
        collection_schedule: config.collection_schedule.get(&ctx.locale).to_string(),
        locale: ctx.locale.clone(),
    }))
}
