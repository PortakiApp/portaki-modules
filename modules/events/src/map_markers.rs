//! Points de ce module pour la carte du livret.
//!
//! Même contrat que dans les autres modules : la plateforme demande, le module rend ses
//! points situés, et le tap d'un marqueur rouvre la surface du module plutôt qu'une fiche
//! dupliquée.
//!
//! Particularité d'`events` : [`resolve_events`] peut aller chercher l'agenda alentour
//! chez OpenAgenda. Le cache KV d'une heure absorbe le cas courant, mais un cache froid
//! coûte un aller-retour réseau dans le budget d'invocation. C'est assumé — et c'est
//! exactement pourquoi la plateforme interroge chaque module séparément, avec sa propre
//! garde : un agenda lent ne doit pas retarder la carte des autres.

use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;
use crate::nearby::resolve_events;

/// Plafond de marqueurs rendus par ce module.
pub const MAX_MARKERS: usize = 50;

#[portaki_sdk::wire]
#[derive(PartialEq)]
pub struct MapMarkersResponse {
    pub markers: Vec<MapMarker>,
}

#[portaki_sdk::query(name = "mapMarkers")]
pub fn map_markers(ctx: Context) -> Result<MapMarkersResponse> {
    let config = ModuleConfig::read(&ctx)?;
    // `false` : ce n'est pas la carte d'accueil, qui ne garde que les prochains jours. La
    // carte montre tout l'agenda connu.
    let events = resolve_events(&ctx, &config, false).unwrap_or_default();
    let markers = events
        .into_iter()
        .filter_map(|event| {
            if !event.has_coords() {
                return None;
            }
            let (lat, lng) = (event.lat?, event.lng?);
            let label = event
                .title
                .pick_with_fallback(&ctx.locale, &ctx.property.locale);
            let mut marker = MapMarker::new(event.id.clone(), lat, lng).kind(MapMarkerKind::Poi);
            if !label.trim().is_empty() {
                marker = marker.label(label);
            }
            Some(marker)
        })
        .take(MAX_MARKERS)
        .collect();
    Ok(MapMarkersResponse { markers })
}
