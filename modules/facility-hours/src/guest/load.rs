//! Load config for guest surfaces.

use portaki_sdk::prelude::*;

use crate::config::{FacilityRow, ModuleConfig};

pub struct GuestData {
    pub facilities: Vec<FacilityRow>,
    pub general_note: String,
    pub locale: String,
    pub property_locale: String,
}

/// The config to show, or `None` when the host has filled in nothing yet.
pub fn load_guest_data(ctx: &GuestContext) -> Result<Option<GuestData>> {
    let config = ModuleConfig::read(ctx)?;
    if config.is_empty() {
        return Ok(None);
    }

    Ok(Some(GuestData {
        facilities: config.parse_facilities(),
        general_note: config
            .general_note
            .pick_with_fallback(&ctx.locale, &ctx.property.locale),
        locale: ctx.locale.clone(),
        property_locale: ctx.property.locale.clone(),
    }))
}
