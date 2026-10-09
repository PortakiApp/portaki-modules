//! Load config for guest surfaces.

use portaki_sdk::prelude::*;

use crate::config::{ModuleConfig, ReviewPlatform};

pub struct GuestData {
    /// Le lien public de l'hôte, quand il se propose à ce séjour.
    pub link: Option<(String, ReviewPlatform)>,
    /// Le voyageur peut écrire à l'hôte, sans le publier.
    pub private_comment: bool,
    pub thank_you: String,
    pub property_name: String,
    /// Prénom de l'hôte, servi par la plateforme ; vide quand elle ne le donne pas.
    pub host_name: String,
    /// La note déjà donnée pour ce séjour. Le formulaire ne se propose pas deux fois (§2.19).
    pub rating_given: Option<u8>,
}

/// What the card shows. The rating is always offered; the public link when the host gave one —
/// to every guest, whatever their rating (spec §3).
pub fn load_guest_data(ctx: &GuestContext) -> Result<GuestData> {
    let config = ModuleConfig::load(ctx)?;
    // Platform the stay was booked on (lowercased, e.g. "airbnb"); `None` on older backends.
    let booking_channel = ctx
        .stay
        .as_ref()
        .and_then(|stay| stay.booking_channel.as_deref());
    let link = config
        .shows_link(booking_channel)
        .then(|| config.public_url().zip(config.platform()))
        .flatten();

    let rating_given = match ctx.stay.as_ref().map(|stay| stay.stay_id) {
        Some(stay_id) => crate::commands::review_for_stay(stay_id)?.map(|review| review.rating),
        None => None,
    };

    Ok(GuestData {
        link,
        private_comment: config.private_comment,
        thank_you: config.thank_you_message.get(&ctx.locale).to_string(),
        property_name: ctx.property.name.clone(),
        host_name: ctx
            .host
            .as_ref()
            .map(|host| host.name.trim().to_string())
            .unwrap_or_default(),
        rating_given,
    })
}
