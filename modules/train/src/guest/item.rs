//! La fiche d'un départ (§2.15).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{Emphasis, KeyValueLayout, SurfaceLevel};
use portaki_sdk::sdui::primitives::{Badge, Button, Card, Grid, KeyValue, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::content::{default_station_distance, Departure, DEFAULT_STATION_LABEL};

/// La largeur d'une tuile (§2.15) : trois tiennent en une rangée sur un téléphone.
const TILE_WIDTH: f64 = 96.0;

pub fn build_item_page(destination: &str, departure: Departure) -> Surface {
    let children: Vec<Component> = vec![
        header(destination, departure),
        measures(departure),
        walk_card(),
        Component::Button(
            Button::new()
                .label("i18n:explore.item.route")
                .action(Action::external(station_maps_url())),
        ),
    ];
    Surface::new(Stack::new().gap(14.0).children(children)).with_id(crate::guest::EXPLORE_ITEM)
}

/// La correspondance en pastille, le trajet en titre, le quai dessous.
fn header(destination: &str, departure: Departure) -> Component {
    Component::Stack(
        Stack::new()
            .gap(6.0)
            .child(Badge::new().label(departure.note.to_string()))
            .child(
                Text::new()
                    .text(format!("{DEFAULT_STATION_LABEL} → {destination}"))
                    .variant(TextVariant::Display),
            )
            .child(
                Text::new()
                    .text(departure.platform.to_string())
                    .variant(TextVariant::Caption)
                    .emphasis(Emphasis::Subtle),
            ),
    )
}

/// Les tuiles : l'heure, le quai, la correspondance. Rien d'autre : une durée de trajet demande
/// un horaire d'arrivée, et le module n'en a pas (§0.1).
fn measures(departure: Departure) -> Component {
    Component::Grid(
        Grid::new()
            .minColumnWidth(TILE_WIDTH)
            .plain(true)
            .children(vec![
                tile(
                    IconName::ClockCircle,
                    "i18n:explore.item.departure",
                    departure.time.to_string(),
                ),
                tile(
                    IconName::Train,
                    "i18n:explore.item.platform",
                    departure.platform.to_string(),
                ),
                tile(
                    IconName::Refresh,
                    "i18n:explore.item.connection",
                    departure.note.to_string(),
                ),
            ]),
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

/// « Aller à la gare » : la distance que l'hôte a renseignée, et de quoi y aller.
fn walk_card() -> Component {
    Component::Card(
        Card::new()
            .surface(SurfaceLevel::Elevated)
            .icon(IconName::MapPin)
            .title("i18n:explore.item.station")
            .child(
                KeyValue::new()
                    .key(DEFAULT_STATION_LABEL)
                    .value(default_station_distance()),
            ),
    )
}

fn station_maps_url() -> String {
    format!(
        "https://www.google.com/maps/search/?api=1&query={}",
        DEFAULT_STATION_LABEL.replace(' ', "+")
    )
}
