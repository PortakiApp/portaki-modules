//! La fiche d'un départ (§2.17).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{Emphasis, KeyValueLayout, SurfaceLevel};
use portaki_sdk::sdui::primitives::{Badge, Button, Card, Grid, KeyValue, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::board::BoardView;
use crate::sncf::Stop;

/// La largeur d'une tuile (§2.17) : trois tiennent en une rangée sur un téléphone.
const TILE_WIDTH: f64 = 96.0;

pub fn build_item_page(view: &BoardView, stop: &Stop) -> Surface {
    let mut children: Vec<Component> = vec![header(view, stop), measures(stop), station_card(view)];
    children.push(Component::Button(
        Button::new()
            .label("i18n:explore.item.route")
            .action(Action::external(station_maps_url(&view.station.label))),
    ));
    Surface::new(Stack::new().gap(14.0).children(children)).with_id(crate::guest::EXPLORE_ITEM)
}

/// Le mode en pastille, le trajet en titre, la mission dessous.
fn header(view: &BoardView, stop: &Stop) -> Component {
    let mut header = Stack::new().gap(6.0);
    if let Some(mode) = stop.mode.as_deref() {
        header = header.child(Badge::new().label(mode.to_string()));
    }
    header = header.child(
        Text::new()
            .text(format!("{} → {}", view.station.label, stop.direction))
            .variant(TextVariant::Display),
    );
    if let Some(headsign) = stop.headsign.as_deref() {
        header = header.child(
            Text::new()
                .text(headsign.to_string())
                .variant(TextVariant::Caption)
                .emphasis(Emphasis::Subtle),
        );
    }
    // Un horaire de fiche horaire n'est pas un horaire temps réel : le dire évite de courir.
    if !stop.realtime {
        header = header.child(
            Text::new()
                .text("i18n:explore.item.scheduled")
                .variant(TextVariant::Caption)
                .emphasis(Emphasis::Subtle),
        );
    }
    Component::Stack(header)
}

/// Les tuiles : l'heure, le jour, le réseau. Pas de quai : l'API n'en donne pas, et l'inventer
/// enverrait le voyageur sur le mauvais.
fn measures(stop: &Stop) -> Component {
    let mut tiles: Vec<Component> = vec![tile(
        IconName::ClockCircle,
        "i18n:explore.item.departure",
        stop.time.clone(),
    )];
    tiles.push(tile(
        IconName::Calendar,
        "i18n:explore.item.day",
        stop.date.clone(),
    ));
    if let Some(network) = stop.network.as_deref().or(stop.mode.as_deref()) {
        tiles.push(tile(
            IconName::Train,
            "i18n:explore.item.network",
            network.to_string(),
        ));
    }
    Component::Grid(
        Grid::new()
            .minColumnWidth(TILE_WIDTH)
            .plain(true)
            .children(tiles),
    )
}

fn tile(icon: IconName, label: &str, value: String) -> Component {
    Component::KeyValue(
        KeyValue::new()
            .layout(KeyValueLayout::Tile)
            .icon(icon)
            .key(label)
            .value(value),
    )
}

/// « Aller à la gare » : son nom officiel, et de quoi y aller.
///
/// ponytail: plus de distance affichée. L'ancienne venait d'une traduction (« 2,3 km » écrit dans
/// `i18n/fr-FR.json`) et valait pour tous les logements du monde. Une vraie distance demande les
/// coordonnées de la gare et celles du logement ; la gare est à un tap de l'itinéraire.
fn station_card(view: &BoardView) -> Component {
    let mut card = Card::new()
        .surface(SurfaceLevel::Elevated)
        .icon(IconName::MapPin)
        .title("i18n:explore.item.station")
        .child(
            KeyValue::new()
                .key("i18n:explore.item.stationName")
                .value(view.station.label.clone()),
        );
    if !view.note.is_empty() {
        card = card.child(
            Text::new()
                .text(view.note.clone())
                .variant(TextVariant::Caption)
                .emphasis(Emphasis::Subtle),
        );
    }
    Component::Card(card)
}

fn station_maps_url(label: &str) -> String {
    format!(
        "https://www.google.com/maps/search/?api=1&query={}",
        label.replace(' ', "+")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_station_name_reaches_the_map_without_spaces() {
        assert_eq!(
            station_maps_url("Gare d'Antibes"),
            "https://www.google.com/maps/search/?api=1&query=Gare+d'Antibes"
        );
    }
}
