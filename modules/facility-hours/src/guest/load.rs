//! Load config for guest surfaces.

use portaki_sdk::prelude::*;

use crate::config::{FacilityRow, ModuleConfig};

pub struct GuestData {
    pub facilities: Vec<FacilityRow>,
    pub general_note: String,
    pub locale: String,
}

/// The config to show, or `None` when the host has filled in nothing yet.
pub fn load_guest_data(ctx: &GuestContext) -> Result<Option<GuestData>> {
    let config = ModuleConfig::load(ctx)?;
    if config.is_empty() {
        return Ok(None);
    }

    Ok(Some(GuestData {
        facilities: config.parse_facilities(),
        general_note: config.general_note.get(&ctx.locale).to_string(),
        locale: ctx.locale.clone(),
    }))
}
