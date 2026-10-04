//! Load config for guest surfaces.

use portaki_sdk::prelude::*;

use crate::config::{FacilityRow, ModuleConfig};

pub struct GuestData {
    pub facilities: Vec<FacilityRow>,
    pub general_note: String,
    pub locale: String,
    /// Le fuseau du logement : un horaire se lit à l'heure du lieu, pas à celle du serveur.
    pub timezone: String,
    /// Arrivée et départ du séjour — données du séjour, jamais saisies par l'hôte (§2.6).
    pub checkin_at: Option<portaki_sdk::prelude::DateTime<portaki_sdk::prelude::Utc>>,
    pub checkout_at: Option<portaki_sdk::prelude::DateTime<portaki_sdk::prelude::Utc>>,
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
        timezone: ctx.timezone.clone(),
        checkin_at: ctx.stay.as_ref().and_then(|stay| stay.checkin_at),
        checkout_at: ctx.stay.as_ref().and_then(|stay| stay.checkout_at),
    }))
}
