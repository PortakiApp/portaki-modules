//! Module queries — la porte de publication et les repères de la carte du livret.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{MapMarker, MapMarkerKind};

use crate::config::ModuleConfig;

/// Ce qu'il faut pour publier : un itinéraire complet — un titre et un niveau.
#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    let mut items = vec![PublishCheck {
        id: "trails".into(),
        level: PublishLevel::Required,
        ok: !config.parse_trails().is_empty(),
        label: crate::i18n::text("publish.trails.label"),
        hint: crate::i18n::text("publish.trails.hint"),
    }];

    // Une ligne sans niveau ne s'affiche pas : l'hôte a saisi un sentier et croit l'avoir publié.
    if config.trails_incomplete() > 0 {
        items.push(PublishCheck {
            id: "level".into(),
            level: PublishLevel::Recommended,
            ok: false,
            label: crate::i18n::text("publish.level.label"),
            hint: crate::i18n::text("publish.level.hint"),
        });
    }

    // Sans durée ni distance, la fiche s'affiche avec moins de tuiles — lisible, mais on ne peut
    // plus choisir entre deux randonnées, ce qui est à quoi sert la liste.
    if config.trails_missing_stats() > 0 {
        items.push(PublishCheck {
            id: "measures".into(),
            level: PublishLevel::Recommended,
            ok: false,
            label: crate::i18n::text("publish.measures.label"),
            hint: crate::i18n::text("publish.measures.hint"),
        });
    }

    Ok(PublishReadiness { items })
}

#[portaki_sdk::wire]
#[derive(PartialEq)]
pub struct MapMarkersResponse {
    pub markers: Vec<MapMarker>,
}

/// Les départs sur la carte du livret (§2.23).
///
/// Un itinéraire sans position n'y figure pas : un repère au hasard vaut moins qu'un repère absent.
#[portaki_sdk::query(name = "mapMarkers", example(label = "Départs sur la carte"))]
pub fn map_markers(ctx: Context) -> Result<MapMarkersResponse> {
    let config = ModuleConfig::load(&ctx)?;
    let markers = config
        .parse_trails()
        .iter()
        .enumerate()
        .filter_map(|(index, trail)| {
            let (lat, lng) = trail.coordinates()?;
            let mut marker = MapMarker::new(format!("trail-{}", trail.route_id(index)), lat, lng)
                .kind(MapMarkerKind::Poi)
                .icon(IconName::Mountain);
            // Pas de constructeur pour `category` dans le SDK : le champ se pose directement. Le
            // livret s'en sert pour la pastille et le sous-titre du lieu, pas pour la famille du
            // filtre — celle-là, il la tire du module.
            marker.category = Some("hiking".into());
            let title = trail.title.get(&ctx.locale).trim().to_string();
            if !title.is_empty() {
                marker = marker.label(title);
            }
            Some(marker)
        })
        .collect();
    Ok(MapMarkersResponse { markers })
}
