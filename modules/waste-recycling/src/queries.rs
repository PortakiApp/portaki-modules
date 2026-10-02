//! Module queries — porte de publication et points pour la carte du livret.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

/// Plafond de marqueurs rendus par ce module.
pub const MAX_MARKERS: usize = 20;

/// Ce qu'il faut pour publier : des bacs **ou** un point d'apport.
///
/// `bins` n'est plus `required` : un gîte rural n'a pas de ramassage devant la porte, et exiger un
/// bac l'empêchait de publier un module qui n'a pourtant rien d'incomplet. Un champ obligatoire ne
/// sait pas dire « l'un ou l'autre » ; cette requête, oui.
#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    let ok = !config.parse_bins().is_empty() || !config.parse_dropoff_points().is_empty();
    Ok(PublishReadiness {
        items: vec![PublishCheck {
            id: "where".into(),
            level: PublishLevel::Required,
            ok,
            label: crate::i18n::text("publish.where.label"),
            hint: crate::i18n::text("publish.where.hint"),
        }],
    })
}

#[portaki_sdk::wire]
#[derive(PartialEq)]
pub struct MapMarkersResponse {
    pub markers: Vec<MapMarker>,
}

/// Les points d'apport sur la carte du livret.
///
/// Le §2.7 l'annonçait et `MAP_SOURCE_MODULES` du livret interroge déjà ce module : il n'avait
/// simplement rien à répondre. Un point sans position n'y figure pas — un repère au hasard vaut
/// moins qu'un repère absent.
#[portaki_sdk::query(name = "mapMarkers", example(label = "Points sur la carte"))]
pub fn map_markers(ctx: Context) -> Result<MapMarkersResponse> {
    let config = ModuleConfig::load(&ctx)?;
    let markers = config
        .parse_dropoff_points()
        .into_iter()
        .filter_map(|point| {
            let (lat, lng) = point.coordinates()?;
            let id = if point.id.trim().is_empty() {
                format!("dropoff-{lat}-{lng}")
            } else {
                point.id.clone()
            };
            let mut marker = MapMarker::new(id, lat, lng).kind(MapMarkerKind::Poi);
            let label = point.title.get(&ctx.locale).trim().to_string();
            if !label.is_empty() {
                marker = marker.label(label);
            }
            Some(marker)
        })
        .take(MAX_MARKERS)
        .collect();
    Ok(MapMarkersResponse { markers })
}
