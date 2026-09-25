//! Load config for guest surfaces — only feasible selected platforms.

use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

pub struct GuestData {
    pub show_airbnb: bool,
    pub show_portaki: bool,
    pub show_qr: bool,
    pub airbnb_url: Option<String>,
    pub thank_you: String,
    pub property_name: String,
}

/// What the card shows, or `None` when no platform is usable for this stay.
pub fn load_guest_data(ctx: &GuestContext) -> Result<Option<GuestData>> {
    let config = ModuleConfig::read(ctx)?;
    // Platform the stay was booked on (lowercased, e.g. "airbnb"); `None` on older backends.
    let booking_channel = ctx
        .stay
        .as_ref()
        .and_then(|stay| stay.booking_channel.as_deref());
    let (show_airbnb, show_portaki) = config.resolve_guest_platforms(booking_channel);
    if !show_airbnb && !show_portaki {
        return Ok(None);
    }

    Ok(Some(GuestData {
        show_airbnb,
        show_portaki,
        show_qr: config.show_qr_code && show_airbnb,
        airbnb_url: config.airbnb_url(),
        thank_you: config
            .thank_you_message
            .pick_with_fallback(&ctx.locale, &ctx.property.locale),
        property_name: ctx.property.name.clone(),
    }))
}
