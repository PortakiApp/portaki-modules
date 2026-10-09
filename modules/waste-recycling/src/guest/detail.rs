//! Guest explore / bottom-sheet detail surface.

use portaki_sdk::prelude::*;
use portaki_sdk::reveal::SECRET_MASK;
use portaki_sdk::sdui::common::{Leading, LeadingVisual, SecretState};
use portaki_sdk::sdui::primitives::{Card, Eyebrow, KeyValue, ListItem, Map, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use super::body::{bin_rows, build_collection_banner, dropoff_row, takeout_note};
use super::load::GuestData;

/// Body-only tree for the bottom sheet (shell supplies header chrome).
pub fn build_detail_surface(data: &GuestData) -> Surface {
    // L'ordre de la maquette : quand la collecte passe, où sont les poubelles, comment aller au
    // local, puis comment trier. On cherche la poubelle sous l'évier avant d'apprendre ce qui va
    // dans le bac jaune (§2.7).
    let mut children = build_collection_banner(data);
    if let Some(card) = inside_card(data) {
        children.push(card);
    }
    if let Some(card) = bin_room_card(data) {
        children.push(card);
    }
    if let Some(card) = bins_card(data) {
        children.push(card);
    }
    if let Some(map) = dropoff_map(data) {
        children.push(map);
    }
    if let Some(card) = dropoff_card(data) {
        children.push(card);
    }
    if let Some(card) = compost_card(data) {
        children.push(card);
    }
    children.extend(takeout_note(data));
    Surface::new(Stack::new().gap(12.0).children(children)).with_id(crate::guest::EXPLORE_DETAIL)
}

/// « Les bacs » : les rangées groupées, comme les deux cartes qui les précèdent.
///
/// En vrac sous la feuille, elles flottaient entre deux cartes encadrées — la maquette les range
/// dans la leur.
fn bins_card(data: &GuestData) -> Option<Component> {
    let rows = bin_rows(data);
    (!rows.is_empty()).then(|| {
        Component::Card(
            Card::new()
                .icon(IconName::Recycle)
                .title("i18n:guest.bins.title")
                .children(rows),
        )
    })
}

/// « Dans le logement » : où chaque bac se trouve, pour les bacs dont l'hôte l'a dit.
fn inside_card(data: &GuestData) -> Option<Component> {
    let rows: Vec<Component> = data
        .bins
        .iter()
        .filter_map(|bin| {
            let location = bin.location.get(&data.locale).trim().to_string();
            if location.is_empty() {
                return None;
            }
            let items = bin.items(&data.locale).join(", ");
            let mut item = ListItem::new()
                .title(location)
                .leading(Leading::Icon("home".into()));
            // L'endroit est le titre, le bac le sous-titre : on cherche « sous l'évier », on y
            // trouve « ordures ménagères ». L'inverse fait lire la consigne avant l'endroit.
            let subtitle = if items.is_empty() {
                bin.title.get(&data.locale).to_string()
            } else {
                format!("{} · {items}", bin.title.get(&data.locale))
            };
            item = item.subtitle(subtitle);
            Some(Component::ListItem(item))
        })
        .collect();
    (!rows.is_empty()).then(|| {
        Component::Card(
            Card::new()
                .icon(IconName::Home)
                .title("i18n:guest.inside.title")
                .children(rows),
        )
    })
}

/// « Le local poubelles » : le code et les heures d'abord, puis le chemin, numéroté.
///
/// Le code et les heures devant parce qu'un local fermé ou verrouillé arrête le trajet avant qu'il
/// commence — descendre trois étages pour lire « fermé le dimanche » en bas, c'est le genre de
/// détail qu'un livret existe pour éviter.
///
/// Rien quand l'hôte n'a rempli aucun des trois : une carte vide promettrait un local.
fn bin_room_card(data: &GuestData) -> Option<Component> {
    if !data.bin_room
        || (data.bin_room_where.is_empty()
            && data.bin_room_steps.is_empty()
            && data.bin_room_code.is_empty()
            && data.bin_room_hours.is_empty())
    {
        return None;
    }
    let mut rows: Vec<Component> = Vec::new();
    if !data.bin_room_where.is_empty() {
        rows.push(Component::Text(
            Text::new()
                .text(data.bin_room_where.clone())
                .variant(TextVariant::Body)
                .emphasis(Emphasis::Strong),
        ));
    }
    if !data.bin_room_code.is_empty() {
        let tile = KeyValue::new().key("i18n:guest.binRoom.code").mono(true);
        // Masqué, la tuile dit quand elle s'ouvrira et ne se copie pas — des points dans le
        // presse-papiers se prendraient pour le code. Révélé, copiable : on le lit devant un
        // digicode.
        rows.push(Component::KeyValue(if data.code_revealed {
            tile.value(data.bin_room_code.clone()).copy(true)
        } else {
            tile.value(SECRET_MASK)
                .secret(SecretState::hidden(data.code_reveal_at.clone()))
        }));
    }
    if !data.bin_room_hours.is_empty() {
        rows.push(Component::KeyValue(
            KeyValue::new()
                .key("i18n:guest.binRoom.hours")
                .value(data.bin_room_hours.clone()),
        ));
    }
    rows.extend(data.bin_room_steps.iter().enumerate().map(|(index, step)| {
        Component::ListItem(
            ListItem::new()
                .title(step.clone())
                .leading(Leading::Visual(Box::new(LeadingVisual {
                    index: Some((index + 1) as u32),
                    ..LeadingVisual::default()
                }))),
        )
    }));
    Some(Component::Card(
        Card::new()
            .icon(IconName::MapPin)
            .title("i18n:guest.binRoom.title")
            .children(rows),
    ))
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
