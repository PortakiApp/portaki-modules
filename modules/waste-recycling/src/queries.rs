//! Module queries — porte de publication et points pour la carte du livret.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

/// Plafond de marqueurs rendus par ce module.
pub const MAX_MARKERS: usize = 20;

/// Ce qui bloque, et ce qui avertit (spec Tri §3).
///
/// Aucune source — ni bac, ni local, ni point d'apport, ni composteur, ni jour de collecte —
/// avertit sans bloquer : la carte ne s'affichera pas, ce qui est le choix de l'hôte. Un gîte
/// rural sans ramassage publie avec ses seuls points d'apport. Les erreurs de champ, elles,
/// bloquent ([`ModuleConfig::problems`]), chacune désignant son champ (`config.<clé>`).
#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    let has_source = !config.collection_days().is_empty()
        || !config.parse_bins().is_empty()
        || !config.parse_dropoff_points().is_empty()
        || config.has_bin_room()
        || config.compost_enabled;

    let mut items = vec![PublishCheck {
        id: "where".into(),
        level: PublishLevel::Recommended,
        ok: has_source,
        label: crate::i18n::text("publish.where.label"),
        hint: crate::i18n::text("publish.where.hint"),
    }];

    // Un bac ramassé devant la porte, mais sans jour : le bandeau ne peut rien en dire (§3).
    if let Some(bin) = config.bin_without_days() {
        let name = bin.title.get("fr").trim().to_string();
        let index = config
            .bins
            .iter()
            .position(|row| row.id == bin.id && row.title == bin.title);
        items.push(PublishCheck {
            id: match index {
                Some(index) => format!("config.bins.{index}.days"),
                None => "collectionDays".into(),
            },
            level: PublishLevel::Recommended,
            ok: false,
            label: crate::i18n::text("host.collection.days"),
            hint: crate::i18n::text_with("publish.days.bin", &[("bin", name.as_str())]),
        });
    }

    items.extend(
        config
            .problems()
            .into_iter()
            .map(|(field, error)| PublishCheck {
                label: crate::i18n::text(field_label(&field)),
                id: format!("config.{field}"),
                level: PublishLevel::Required,
                ok: false,
                hint: error,
            }),
    );

    Ok(PublishReadiness { items })
}

/// Le libellé du champ en défaut : celui du formulaire, sans l'index de la ligne.
fn field_label(field: &str) -> &'static str {
    match field {
        "bins" => "host.bins.title",
        "dropoff_points" => "host.dropoff.title",
        "bin_room_where" => "host.binRoom.where",
        "bin_room_code" => "host.binRoom.code",
        "compost_location" => "host.compost.location",
        _ if field.starts_with("bins.") && field.ends_with(".title") => "host.bin.title",
        _ if field.starts_with("bins.") => "host.bin.items",
        _ if field.ends_with(".title") => "host.dropoff.pointTitle",
        _ if field.ends_with(".lat") => "host.dropoff.position",
        _ => "host.dropoff.accepts",
    }
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
