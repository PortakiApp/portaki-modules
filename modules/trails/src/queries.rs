//! Module queries — la porte de publication et les repères de la carte du livret.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{MapMarker, MapMarkerKind};

use crate::config::{ModuleConfig, TrailRow};

/// Ce qu'il faut pour publier : un itinéraire complet — un titre et un niveau.
#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    let property = ctx.property.coordinates.map(|point| (point.lat, point.lng));
    let mut items = vec![PublishCheck {
        id: "trails".into(),
        level: PublishLevel::Required,
        ok: !config.parse_trails().is_empty(),
        label: crate::i18n::text("publish.trails.label"),
        hint: crate::i18n::text("publish.trails.hint"),
    }];

    // Les erreurs du formulaire bloquent, chacune sur son champ — un niveau manquant compris
    // (une ligne sans niveau ne s'affiche pas, l'hôte croirait l'avoir publiée), et un départ
    // posé à plus de 100 km du logement.
    items.extend(
        config
            .problems(property)
            .into_iter()
            .map(|(field, error)| PublishCheck {
                label: crate::i18n::text(field_label(&field)),
                id: format!("config.{field}"),
                level: PublishLevel::Required,
                ok: false,
                hint: error,
            }),
    );

    // Sans départ sur la carte : ni repère, ni « itinéraire jusqu'au départ ». La spec l'exige ;
    // des itinéraires existants n'en ont pas et s'affichent quand même — on avertit.
    if let Some(index) = config.trails.iter().position(TrailRow::start_missing) {
        items.push(PublishCheck {
            id: format!("config.trails.{index}.lat"),
            level: PublishLevel::Recommended,
            ok: false,
            label: crate::i18n::text("host.trails.start"),
            hint: crate::i18n::text("host.trails.start.required"),
        });
    }

    // Une trace refusée ne s'ignore plus en silence : le livret ne la dessinerait pas, et le
    // voyageur téléchargerait un fichier que son application rejette. Une trace que la plateforme
    // ne sert pas (retirée, au-delà de ce qu'elle lit) ne se juge pas ici.
    items.extend(
        config
            .trails
            .iter()
            .enumerate()
            .filter(|(_, trail)| !trail.is_blank())
            .filter_map(|(index, trail)| {
                let bytes = portaki_sdk::host::files::read(trail.gpx_ref()?).ok()?;
                let refusal = crate::gpx::refusal(&bytes)?;
                Some(PublishCheck {
                    id: format!("config.trails.{index}.gpx_file"),
                    level: PublishLevel::Required,
                    ok: false,
                    label: crate::i18n::text("host.trails.gpx"),
                    hint: crate::i18n::text(refusal),
                })
            }),
    );

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

    // La page publique n'empêche jamais de publier le livret : hors bornes, son bloc se masque
    // (moins de deux) ou se coupe (plus de quatre), et l'hôte en est averti.
    let chosen = config.public_chosen().len();
    if config.public_enabled
        && !(crate::config::PUBLIC_MIN..=crate::config::PUBLIC_MAX).contains(&chosen)
    {
        items.push(PublishCheck {
            id: "config.public_enabled".into(),
            level: PublishLevel::Recommended,
            ok: false,
            label: crate::i18n::text("host.public.title"),
            hint: crate::i18n::text("publish.public.hint"),
        });
    }

    Ok(PublishReadiness { items })
}

/// Le libellé du champ en défaut : celui du formulaire, sans l'index de la ligne.
fn field_label(field: &str) -> &'static str {
    match field.rsplit('.').next().unwrap_or_default() {
        "title" => "host.trails.rowTitle",
        "level" => "host.level.label",
        "duration_min" => "host.trails.duration",
        "distance_km" => "host.trails.distance",
        "elevation_m" => "host.trails.elevation",
        "link_url" => "host.trails.link",
        "description" => "host.trails.description",
        "commune_url" => "host.commune.label",
        "lat" => "host.trails.start",
        "season_from" => "host.trails.season.from",
        "season_to" => "host.trails.season.to",
        _ => "publish.trails.label",
    }
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
