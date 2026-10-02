//! La liste par niveau, et la fiche d'un itinéraire.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{
    Emphasis, KeyValueLayout, Leading, MapInteractionMode, MapMarker, MapMarkerKind, MapViewport,
    SurfaceLevel,
};
use portaki_sdk::sdui::primitives::{
    Badge, Button, Card, Grid, KeyValue, Link, ListItem, Map, Stack, Text,
};
use portaki_sdk::sdui::surface::Surface;

use super::load::GuestData;
use super::rows::{levels_with_counts, trail_row};
use crate::config::TrailRow;
use crate::format;

/// La largeur d'une tuile du dessin (§4.2) : quatre mesures tiennent en deux rangées sur un
/// téléphone, en une sur un écran large.
const TILE_WIDTH: f64 = 136.0;

/// La liste : une carte par niveau, puis le lien de la commune.
pub fn build_trails_page(data: &GuestData) -> Surface {
    let mut children: Vec<Component> = vec![header(
        "i18n:guest.page.title",
        count_subtitle(data.trails.len()),
    )];

    for (level, count) in levels_with_counts(&data.trails) {
        let rows: Vec<Component> = data
            .trails
            .iter()
            .enumerate()
            .filter(|(_, trail)| trail.level_key() == Some(level))
            // La forme ici, pas sur la carte d'accueil : la liste est l'endroit où l'on compare.
            .map(|(index, trail)| trail_row(data, trail, index, true))
            .collect();
        children.push(
            Card::new()
                .icon(IconName::Mountain)
                .title(level_heading(level, count))
                .children(rows)
                .into(),
        );
    }

    if let Some(url) = &data.commune_url {
        children.push(
            Link::new()
                .label("i18n:guest.commune.label")
                .href(url.clone())
                .action(Action::external(url.clone()))
                .into(),
        );
    }

    Surface::new(Stack::new().gap(14.0).children(children)).with_id(crate::guest::EXPLORE_DETAIL)
}

/// La fiche : niveau, titre, départ, quatre tuiles, description, plan, « Avant de partir », actions.
pub fn build_trail_detail(data: &GuestData, trail: &TrailRow) -> Surface {
    let mut children: Vec<Component> = vec![trail_header(data, trail)];

    if let Some(tiles) = measures(trail) {
        children.push(tiles);
    }

    let description = trail.description.get(&data.locale).trim().to_string();
    if !description.is_empty() {
        children.push(
            Text::new()
                .text(description)
                .variant(TextVariant::Body)
                .into(),
        );
    }

    if let Some(map) = start_map(data, trail) {
        children.push(map);
    }

    children.push(before_you_go());
    children.extend(actions(data, trail));

    Surface::new(Stack::new().gap(14.0).children(children)).with_id(crate::guest::EXPLORE_ITEM)
}

fn header(title: &str, subtitle: String) -> Component {
    Stack::new()
        .gap(4.0)
        .children(vec![
            Text::new().text(title).variant(TextVariant::Title).into(),
            Text::new()
                .text(subtitle)
                .variant(TextVariant::Caption)
                .emphasis(Emphasis::Subtle)
                .into(),
        ])
        .into()
}

/// Le niveau au-dessus du titre, et sous le titre où l'on part.
fn trail_header(data: &GuestData, trail: &TrailRow) -> Component {
    let mut children: Vec<Component> = Vec::new();
    if let Some(level) = trail.level_key() {
        children.push(Badge::new().label(format::level(level)).into());
    }
    children.push(
        Text::new()
            .text(data.title(trail))
            .variant(TextVariant::Display)
            .into(),
    );
    if let Some(line) = start_line(data, trail) {
        children.push(
            Text::new()
                .text(line)
                .variant(TextVariant::Caption)
                .emphasis(Emphasis::Subtle)
                .into(),
        );
    }
    Stack::new().gap(6.0).children(children).into()
}

/// « Départ : devant le logement », « Départ : Plage de la Garoupe · 400 m », ou l'adresse seule.
///
/// Pas de « 15 min en voiture » : il faudrait un calcul d'itinéraire, et une durée inventée sur une
/// ligne droite serait fausse là où elle compte. Le bouton d'itinéraire dit le reste.
fn start_line(data: &GuestData, trail: &TrailRow) -> Option<String> {
    if data.starts_at_property(trail) {
        return Some(
            t!("guest.start.atProperty").unwrap_or_else(|_| "i18n:guest.start.atProperty".into()),
        );
    }
    let address = trail.address.as_deref().map(str::trim).unwrap_or_default();
    if address.is_empty() {
        return None;
    }
    let metres = data.start_metres(trail);
    match metres {
        Some(metres) if metres < crate::config::FAR_START_METRES => t!(
            "guest.start.near",
            place = address,
            distance = format::walking(metres)
        )
        .ok(),
        _ => t!("guest.start", place = address).ok(),
    }
}

/// Les quatre tuiles. Celles dont l'hôte n'a pas la mesure ne s'affichent pas.
fn measures(trail: &TrailRow) -> Option<Component> {
    let mut tiles: Vec<Component> = Vec::new();
    if let Some(minutes) = trail.duration() {
        tiles.push(tile(
            IconName::ClockCircle,
            "i18n:guest.tile.duration",
            format::duration(minutes),
        ));
    }
    if let Some(km) = trail.distance() {
        // ponytail: `compass` n'est pas dans `IconName` (9.5.1) — `gauge` est le cadran le plus
        // proche. À remplacer si la boussole entre au vocabulaire.
        tiles.push(tile(
            IconName::Gauge,
            "i18n:guest.tile.distance",
            format::distance(km),
        ));
    }
    if let Some(metres) = trail.elevation() {
        // ponytail: pas d'`arrow-up` non plus ; la montagne dit le dénivelé sans flèche.
        tiles.push(tile(
            IconName::Mountain,
            "i18n:guest.tile.elevation",
            format::elevation(metres),
        ));
    }
    if let Some(shape) = trail.shape_key() {
        tiles.push(tile(
            IconName::Refresh,
            "i18n:guest.tile.shape",
            format::shape(shape),
        ));
    }
    (!tiles.is_empty()).then(|| {
        Grid::new()
            .minColumnWidth(TILE_WIDTH)
            .plain(true)
            .children(tiles)
            .into()
    })
}

fn tile(icon: IconName, label: &str, value: String) -> Component {
    KeyValue::new()
        .layout(KeyValueLayout::Tile)
        .icon(icon)
        .key(label)
        .value(value)
        .into()
}

/// Le plan, centré sur le départ quand on le connaît, sur le logement sinon.
///
/// Un seul repère quand la randonnée part du logement : deux épingles au même endroit disent qu'il
/// y a deux endroits. Le tracé attend `Map.path` et le dépôt du GPX.
fn start_map(data: &GuestData, trail: &TrailRow) -> Option<Component> {
    let start = trail.coordinates();
    let (center_lat, center_lng) = start.or(data.property)?;

    let mut markers: Vec<MapMarker> = Vec::new();
    if let Some((lat, lng)) = data.property {
        markers.push(
            MapMarker::new("property", lat, lng)
                .label("i18n:guest.map.property")
                .kind(MapMarkerKind::Property),
        );
    }
    if let Some((lat, lng)) = start.filter(|_| !data.starts_at_property(trail)) {
        markers.push(
            MapMarker::new("start", lat, lng)
                .label("i18n:guest.map.start")
                .icon(IconName::Mountain)
                .kind(MapMarkerKind::Poi),
        );
    }
    if markers.is_empty() {
        return None;
    }

    Some(
        Map::new()
            .viewport(MapViewport::new(center_lat, center_lng, Some(13.0)))
            .markers(markers)
            .label("i18n:guest.map.label")
            .isStatic(true)
            .interactionMode(MapInteractionMode::None)
            .into(),
    )
}

/// « Avant de partir » : l'eau et la météo (§2.23).
///
/// Deux lignes écrites par le module, pas par l'hôte : elles valent pour tout sentier, et un hôte
/// qui les oublierait ne rendrait pas la randonnée moins exposée.
fn before_you_go() -> Component {
    Card::new()
        .surface(SurfaceLevel::Elevated)
        .icon(IconName::InfoCircle)
        .title("i18n:guest.before.title")
        .children(vec![
            ListItem::new()
                .title("i18n:guest.before.water")
                .leading(Leading::Icon("droplet".into()))
                .into(),
            ListItem::new()
                .title("i18n:guest.before.weather")
                .leading(Leading::Icon("cloud-sun".into()))
                .into(),
        ])
        .into()
}

/// La barre du bas : « Ouvrir la trace » puis « Itinéraire jusqu'au départ ».
///
/// Sans fiche tierce, l'itinéraire devient l'action principale — c'est le cas « sans trace » du
/// §2.23, et il est la v1 : le dépôt du GPX attend le lot de stockage de fichiers.
fn actions(data: &GuestData, trail: &TrailRow) -> Vec<Component> {
    let mut bar: Vec<Component> = Vec::new();
    if let Some(url) = trail.link() {
        bar.push(
            Button::new()
                .label("i18n:guest.openTrace")
                .action(Action::external(url.to_string()))
                .into(),
        );
    }
    if let Some((lat, lng)) = trail.coordinates().filter(|_| data.starts_far(trail)) {
        let mut button = Button::new()
            .label("i18n:guest.routeToStart")
            .action(Action::external(google_maps_url(lat, lng)));
        if !bar.is_empty() {
            button = button.variant(ButtonVariant::Outline);
        }
        bar.push(button.into());
    }
    bar
}

fn google_maps_url(lat: f64, lng: f64) -> String {
    format!("https://www.google.com/maps/search/?api=1&query={lat},{lng}")
}

/// « ▲ Facile · 3 » en tête d'une carte de la liste.
fn level_heading(level: &str, count: usize) -> String {
    let label = format::level(level);
    t!("guest.level.count", label = label.clone(), count = count)
        .unwrap_or_else(|_| format!("{label} · {count}"))
}

fn count_subtitle(count: usize) -> String {
    let key = if count == 1 {
        "guest.page.subtitle.one"
    } else {
        "guest.page.subtitle"
    };
    t!(key, count = count).unwrap_or_else(|_| format!("i18n:{key}"))
}
