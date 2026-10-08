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
    /// Prénom de l'hôte, servi par la plateforme ; vide quand elle ne le donne pas.
    pub host_name: String,
    /// La note déjà donnée pour ce séjour. Le formulaire ne se propose pas deux fois (§2.19).
    pub rating_given: Option<u8>,
}

/// What the card shows, or `None` when no platform is usable for this stay.
pub fn load_guest_data(ctx: &GuestContext) -> Result<Option<GuestData>> {
    let config = ModuleConfig::load(ctx)?;
    // Platform the stay was booked on (lowercased, e.g. "airbnb"); `None` on older backends.
    let booking_channel = ctx
        .stay
        .as_ref()
        .and_then(|stay| stay.booking_channel.as_deref());
    let (show_airbnb, show_portaki) = config.resolve_guest_platforms(booking_channel);
    if !show_airbnb && !show_portaki {
        return Ok(None);
    }

    let rating_given = match ctx.stay.as_ref().map(|stay| stay.stay_id) {
        Some(stay_id) => crate::commands::review_for_stay(stay_id)?.map(|review| review.rating),
        None => None,
    };

    Ok(Some(GuestData {
        show_airbnb,
        show_portaki,
        show_qr: config.show_qr_code && show_airbnb,
        airbnb_url: config.airbnb_url(),
        thank_you: config.thank_you_message.get(&ctx.locale).to_string(),
        property_name: ctx.property.name.clone(),
        host_name: ctx
            .host
            .as_ref()
            .map(|host| host.name.trim().to_string())
            .unwrap_or_default(),
        rating_given,
    }))
}
