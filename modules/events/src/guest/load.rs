//! Load config + nearby events for guest surfaces.

use portaki_sdk::prelude::*;

use crate::config::{EventRow, ModuleConfig};
use crate::nearby::{nearby_ready, resolve_events};

pub struct GuestData {
    pub events: Vec<EventRow>,
    pub disclaimer: String,
    pub locale: String,
    pub property_locale: String,
    pub show_map: bool,
}

/// The events to show, or `None` when there is nothing: no event from the host, and no nearby
/// search possible (off, no key, or a property without a position).
pub fn load_guest_data(ctx: &GuestContext, surface_id: SurfaceId) -> Result<Option<GuestData>> {
    let config = ModuleConfig::read(ctx)?;
    let for_home = surface_id == crate::ids::HOME_CARD;
    let events = resolve_events(ctx, &config, for_home)?;

    let has_manual = !config.parse_events().is_empty();
    if events.is_empty() && (for_home || (!has_manual && !nearby_ready(ctx, &config))) {
        return Ok(None);
    }

    let show_map =
        surface_id == crate::ids::EXPLORE_DETAIL && events.iter().any(|e| e.has_coords());

    Ok(Some(GuestData {
        events,
        disclaimer: config
            .disclaimer
            .pick_with_fallback(&ctx.locale, &ctx.property.locale),
        locale: ctx.locale.clone(),
        property_locale: ctx.property.locale.clone(),
        show_map,
    }))
}
