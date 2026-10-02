//! Guest explore / bottom-sheet detail surface.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Eyebrow, ListItem, Map, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use super::body::{build_bins_body, dropoff_row};
use super::load::GuestData;

/// Body-only tree for the bottom sheet (shell supplies header chrome).
pub fn build_detail_surface(data: &GuestData) -> Surface {
    let mut children = build_bins_body(data, true);
    if let Some(map) = dropoff_map(data) {
        children.push(map);
    }
    if let Some(card) = dropoff_card(data) {
        children.push(card);
    }
    if let Some(card) = compost_card(data) {
        children.push(card);
    }
    Surface::new(Stack::new().gap(12.0).children(children)).with_id(crate::guest::EXPLORE_DETAIL)
}

/// Le plan, centré sur le logement, avec un repère par point situé (§3.2).
///
/// Sans position du logement il n'y a pas de cadrage : on montre les points en liste et rien de
/// plus, plutôt qu'un plan centré sur une coordonnée inventée.
fn dropoff_map(data: &GuestData) -> Option<Component> {
    let (lat, lng) = data.property?;
    let mut markers = vec![MapMarker::new("property", lat, lng)
        .label("i18n:guest.map.property")
        .kind(MapMarkerKind::Property)];
    for (index, point) in data.dropoff_points.iter().enumerate() {
        if let Some((point_lat, point_lng)) = point.coordinates() {
            let mut marker = MapMarker::new(format!("dropoff-{index}"), point_lat, point_lng)
                .kind(MapMarkerKind::Poi);
            let label = point.title.get(&data.locale).trim().to_string();
            if !label.is_empty() {
                marker = marker.label(label);
            }
            markers.push(marker);
        }
    }
    if markers.len() == 1 {
        return None;
    }
    Some(Component::Map(
        Map::new()
            .viewport(MapViewport::new(lat, lng, Some(13.0)))
            .markers(markers)
            .label("i18n:guest.map.label")
            .isStatic(true)
            .interactionMode(MapInteractionMode::None),
    ))
}

/// Les points d'apport, chacun ouvrant son itinéraire.
fn dropoff_card(data: &GuestData) -> Option<Component> {
    if data.dropoff_points.is_empty() {
        return None;
    }
    let rows: Vec<Component> = data
        .dropoff_points
        .iter()
        .map(|point| {
            let row = dropoff_row(data, point);
            // Toucher la rangée ouvre l'itinéraire (§3.2) — seulement quand on sait où aller.
            match (point.coordinates(), row) {
                (Some((lat, lng)), Component::ListItem(item)) => Component::ListItem(
                    item.action(Action::external(google_maps_url(lat, lng)))
                        .chevron(true),
                ),
                (_, row) => row,
            }
        })
        .collect();
    Some(Component::Card(
        Card::new()
            .icon(IconName::MapPin)
            .title("i18n:guest.dropoff.title")
            .children(rows),
    ))
}

/// Le composteur : où il est, ce qui y va et ce qui n'y va pas.
fn compost_card(data: &GuestData) -> Option<Component> {
    let location = data.compost_location.trim();
    if location.is_empty() {
        return None;
    }
    let mut children: Vec<Component> = vec![Component::Text(
        Text::new()
            .text(location)
            .variant(TextVariant::Body)
            .emphasis(Emphasis::Strong),
    )];
    for (key, items) in [
        ("guest.compost.accepted", &data.compost_accepted),
        ("guest.compost.refused", &data.compost_refused),
    ] {
        if items.is_empty() {
            continue;
        }
        children.push(Component::Eyebrow(
            Eyebrow::new().text(format!("i18n:{key}")),
        ));
        for item in items {
            children.push(Component::ListItem(ListItem::new().title(item.clone())));
        }
    }
    Some(Component::Card(
        Card::new()
            .icon(IconName::Recycle)
            .title("i18n:guest.compost.title")
            .children(children),
    ))
}

fn google_maps_url(lat: f64, lng: f64) -> String {
    format!("https://www.google.com/maps/search/?api=1&query={lat},{lng}")
}
