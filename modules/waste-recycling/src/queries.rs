//! Module queries — porte de publication et points pour la carte du livret.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

/// Plafond de marqueurs rendus par ce module.
pub const MAX_MARKERS: usize = 20;

/// Ce qu'il faut pour publier : une source de déchets — des jours de collecte, des bacs, ou un
/// point d'apport (§10).
///
/// `bins` n'est plus `required` : un gîte rural n'a pas de ramassage devant la porte, et exiger un
/// bac l'empêchait de publier un module qui n'a pourtant rien d'incomplet. Un champ obligatoire ne
/// sait pas dire « l'un ou l'autre » ; cette requête, oui.
#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    let has_source = !config.collection_days().is_empty()
        || !config.parse_bins().is_empty()
        || !config.parse_dropoff_points().is_empty();

    let mut items = vec![PublishCheck {
        id: "where".into(),
        level: PublishLevel::Required,
        ok: has_source,
        label: crate::i18n::text("publish.where.label"),
        hint: crate::i18n::text("publish.where.hint"),
    }];

    // Un point d'apport sans rien d'accepté s'affiche sans dire ce qu'on y dépose — la ligne existe
    // et ne sert à rien. Recommandé et non requis : le reste du module marche.
    if config.dropoff_points_missing_accepts() > 0 {
        items.push(PublishCheck {
            id: "dropoffAccepts".into(),
            level: PublishLevel::Recommended,
            ok: false,
            label: crate::i18n::text("publish.accepts.label"),
            hint: crate::i18n::text("publish.accepts.hint"),
        });
    }

    Ok(PublishReadiness { items })
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
