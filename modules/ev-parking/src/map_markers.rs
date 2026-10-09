//! La borne sur la Carte du livret (spec Parking VE §7).

use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

#[portaki_sdk::wire]
#[derive(PartialEq)]
pub struct MapMarkersResponse {
    pub markers: Vec<MapMarker>,
}

/// Un repère ⚡ à la position de la borne. Sans position, rien : un repère au hasard vaut moins
/// qu'un repère absent.
#[portaki_sdk::query(name = "mapMarkers", example(label = "La borne sur la carte"))]
pub fn map_markers(ctx: Context) -> Result<MapMarkersResponse> {
    let config = ModuleConfig::load(&ctx)?;
    let markers = config
        .charger_lat
        .zip(config.charger_lng)
        .map(|(lat, lng)| {
            let label = config
                .spot_text(&ctx.locale)
                .map(str::to_string)
                .unwrap_or_else(|| "i18n:guest.charger".to_string());
            MapMarker::new("charger", lat, lng)
                .kind(MapMarkerKind::Poi)
                .label(label)
        })
        .into_iter()
        .collect();
    Ok(MapMarkersResponse { markers })
}
