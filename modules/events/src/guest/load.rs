//! Load config + nearby events for guest surfaces.

use portaki_sdk::prelude::*;

use crate::config::{EventRow, ModuleConfig};
use crate::nearby::{host_now, resolve_events};

pub struct GuestData {
    pub events: Vec<EventRow>,
    pub disclaimer: String,
    pub locale: String,
    pub show_map: bool,
    /// Le repère du logement, pour la marche et le plan de la fiche.
    pub property_coords: Option<(f64, f64)>,
    /// L'heure et le fuseau du logement, pour le jour d'un événement et le badge « Ce soir ».
    pub now: chrono::DateTime<chrono::Utc>,
    pub tz: Option<portaki_sdk::host::time::PropertyTz>,
}

impl GuestData {
    pub fn is_tonight(&self, event: &EventRow) -> bool {
        crate::time_format::is_tonight(event, self.now, self.tz.as_ref())
    }

    /// « Demain à 20:30 », « à 20:30 » aujourd'hui — voir [`crate::time_format::event_when`].
    pub fn when(&self, event: &EventRow) -> String {
        crate::time_format::event_when(event, self.tz.as_ref(), self.now, &self.locale)
    }

    /// La clé du badge du jour, pour un événement d'aujourd'hui pas encore fini.
    pub fn today_key(&self, event: &EventRow) -> Option<&'static str> {
        self.is_tonight(event)
            .then(|| crate::time_format::today_key(event, self.tz.as_ref()))
    }
}

/// The events to show, or `None` when there is none in the stay window — the guest then reads
/// the empty state (§9 #1) rather than an empty list.
pub fn load_guest_data(ctx: &GuestContext, surface_id: SurfaceId) -> Result<Option<GuestData>> {
    let config = ModuleConfig::load(ctx)?;
    // The upcoming card headlines the *next* event: like the home card, past ones are dropped.
    let for_home =
        surface_id == crate::guest::HOME_CARD || surface_id == crate::guest::UPCOMING_CARD;
    let events = resolve_events(ctx, &config, for_home)?;

    if events.is_empty() {
        return Ok(None);
    }

    let show_map =
        surface_id == crate::guest::EXPLORE_DETAIL && events.iter().any(|e| e.has_coords());

    Ok(Some(GuestData {
        events,
        disclaimer: config.disclaimer.get(&ctx.locale).to_string(),
        locale: ctx.locale.clone(),
        show_map,
        property_coords: ctx
            .property
            .coordinates
            .as_ref()
            .map(|point| (point.lat, point.lng))
            .filter(|(lat, lng)| *lat != 0.0 || *lng != 0.0),
        now: host_now(),
        tz: ctx.property_tz(),
    }))
}
