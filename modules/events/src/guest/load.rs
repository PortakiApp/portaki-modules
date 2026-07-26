//! Load config + nearby events for guest surfaces.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use crate::config::{load_config, EventRow, ModuleConfig};
use crate::nearby::{has_open_agenda, resolve_events};

use super::empty::{empty_content_state, empty_state_if_module_not_ready};

pub struct GuestData {
    pub events: Vec<EventRow>,
    pub disclaimer: String,
    pub locale: String,
    pub property_locale: String,
    pub show_map: bool,
}

pub enum GuestLoad {
    Ready(GuestData),
    Empty(Box<Surface>),
}

pub fn load_guest_data(ctx: &GuestContext, surface_id: SurfaceId) -> Result<GuestLoad> {
    if let Some(surface) = empty_state_if_module_not_ready(surface_id)? {
        return Ok(GuestLoad::Empty(Box::new(surface)));
    }

    let config = load_config().unwrap_or_else(|_| ModuleConfig::default());
    let for_home = surface_id == crate::ids::HOME_CARD;
    let events = resolve_events(ctx, &config, for_home)?;

    let has_manual = !config.parse_events().is_empty();
    let nearby_ready = config.nearby_enabled && has_open_agenda(ctx);
    if events.is_empty() && !has_manual && !nearby_ready {
        return Ok(GuestLoad::Empty(Box::new(empty_content_state(surface_id))));
    }
    if events.is_empty() && for_home {
        return Ok(GuestLoad::Empty(Box::new(empty_content_state(surface_id))));
    }

    let show_map =
        surface_id == crate::ids::EXPLORE_DETAIL && events.iter().any(|e| e.has_coords());

    Ok(GuestLoad::Ready(GuestData {
        events,
        disclaimer: config
            .disclaimer
            .pick_with_fallback(&ctx.locale, &ctx.property.locale),
        locale: ctx.locale.clone(),
        property_locale: ctx.property.locale.clone(),
        show_map,
    }))
}
