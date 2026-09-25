//! Load config for guest surfaces.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use crate::config::{FacilityRow, ModuleConfig};

use super::empty::{empty_content_state, empty_state_if_module_not_ready};

pub struct GuestData {
    pub facilities: Vec<FacilityRow>,
    pub general_note: String,
    pub locale: String,
    pub property_locale: String,
}

pub enum GuestLoad {
    Ready(GuestData),
    Empty(Box<Surface>),
}

pub fn load_guest_data(ctx: &GuestContext, surface_id: SurfaceId) -> Result<GuestLoad> {
    if let Some(surface) = empty_state_if_module_not_ready(surface_id)? {
        return Ok(GuestLoad::Empty(Box::new(surface)));
    }

    let config = ModuleConfig::read(ctx)?;
    if config.is_empty() {
        return Ok(GuestLoad::Empty(Box::new(empty_content_state(surface_id))));
    }

    Ok(GuestLoad::Ready(GuestData {
        facilities: config.parse_facilities(),
        general_note: config
            .general_note
            .pick_with_fallback(&ctx.locale, &ctx.property.locale),
        locale: ctx.locale.clone(),
        property_locale: ctx.property.locale.clone(),
    }))
}
