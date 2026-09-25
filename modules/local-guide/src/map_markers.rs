//! Points de ce module pour la carte du livret.
//!
//! La plateforme appelle cette requête quand le voyageur ouvre la carte, agrège la réponse
//! avec celle des autres modules et avec son propre catalogue de POI. Le module, lui, ne
//! sait pas qui l'interroge et n'écrit nulle part : il rend ses bons plans situés, et rien
//! d'autre. La fiche détaillée reste rendue par sa propre surface, au tap du marqueur —
//! dupliquer ici le modèle de fiche de la plateforme ne servirait personne.

use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

/// Plafond de marqueurs rendus par ce module.
///
/// La plateforme pose le sien de son côté ; celui-ci est la même limite vue du module, pour
/// qu'aucune configuration d'hôte ne puisse le rendre bavard sans qu'on l'ait décidé ici.
/// Six créneaux de saisie aujourd'hui, donc large — c'est une butée, pas un quota.
pub const MAX_MARKERS: usize = 50;

#[portaki_sdk::wire]
#[derive(PartialEq)]
pub struct MapMarkersResponse {
    pub markers: Vec<MapMarker>,
}

#[portaki_sdk::query(name = "mapMarkers", example(label = "Points sur la carte"))]
pub fn map_markers(ctx: Context) -> Result<MapMarkersResponse> {
    let config = ModuleConfig::load(&ctx)?;
    let markers = config
        .parse_spots()
        .into_iter()
        .filter_map(|spot| {
            // Les spots sans position ne sont pas une anomalie : la carte est arrivée après
            // eux, et l'hôte n'est pas obligé de la remplir.
            let (lat, lng) = spot.coords()?;
            let label = spot.title.get(&ctx.locale).to_string();
            let mut marker = MapMarker::new(spot.id.clone(), lat, lng).kind(MapMarkerKind::Poi);
            if !label.trim().is_empty() {
                marker = marker.label(label);
            }
            // `category` est un champ du marqueur, là où `label` et `kind` ont un setter.
            if let Some(category) = spot.category.as_deref().filter(|c| !c.trim().is_empty()) {
                marker.category = Some(category.to_string());
            }
            Some(marker)
        })
        .take(MAX_MARKERS)
        .collect();
    Ok(MapMarkersResponse { markers })
}
