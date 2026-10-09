//! La fiche d'un départ (§2.17).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{Emphasis, KeyValueLayout, SurfaceLevel, Tone};
use portaki_sdk::sdui::primitives::{Badge, Button, Card, Grid, InfoBanner, KeyValue, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::board::BoardView;
use crate::sncf::{Stop, Way};

/// La largeur d'une tuile (§2.17) : trois tiennent en une rangée sur un téléphone.
const TILE_WIDTH: f64 = 96.0;

pub fn build_item_page(view: &BoardView, stop: &Stop, way: Way) -> Surface {
    let mut children: Vec<Component> = vec![header(view, stop, way)];
    if stop.cancelled {
        children.push(Component::InfoBanner(
            InfoBanner::new()
                .tone(Tone::Danger)
                .title("i18n:explore.item.cancelled.title")
                .message("i18n:explore.item.cancelled.message"),
        ));
    } else if let Some(late) = stop.delay_min {
        children.push(delay_banner(stop, late));
    }
    children.extend([measures(view, stop), station_card(view)]);
    children.push(Component::Button(
        Button::new()
            .label("i18n:explore.item.route")
            .icon(IconName::MapPin)
            .action(Action::external(station_maps_url(&view.station.label))),
    ));
    // Le billet s'achète chez l'opérateur, et c'est le même que celui qui sert les horaires : le
    // lien n'ajoute pas de fournisseur, il ouvre celui qui est déjà derrière ce tableau.
    children.push(Component::Button(
        Button::new()
            .label("i18n:explore.item.ticket")
            .variant(ButtonVariant::Outline)
            .action(Action::external(TICKET_URL)),
    ));
    Surface::new(Stack::new().gap(14.0).children(children)).with_id(crate::guest::EXPLORE_ITEM)
}

/// L'état en pastille, le trajet en titre, la mission dessous.
fn header(view: &BoardView, stop: &Stop, way: Way) -> Component {
    let mut header = Stack::new().gap(6.0);
    if let Some(badge) = crate::guest::detail::status_badge(stop) {
        header = header.child(Badge::new().label(badge.label).tone(badge.tone));
    }
    // Le sens compte dans la flèche : en « vers la gare », le train vient de la destination et
    // arrive au logement. L'écrire à l'envers envoyait le voyageur dans le mauvais train.
    let (from, to) = match way {
        Way::From => (&view.station.label, &stop.direction),
        Way::To => (&stop.direction, &view.station.label),
    };
    header = header.child(
        Text::new()
            .text(format!("{from} → {to}"))
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
fn measures(view: &BoardView, stop: &Stop) -> Component {
    let mut tiles: Vec<Component> = vec![tile(
        IconName::ClockCircle,
        "i18n:explore.item.departure",
        stop.time.clone(),
    )];
    // « Aujourd'hui » ou « Demain », pas `2026-10-04`. La tuile servait la date de l'API telle
    // quelle, au format ISO, à un voyageur qui lit dix langues. Le tableau ne couvre que ces deux
    // jours — §0.7 borne les données à la veille et au lendemain du séjour — donc deux mots
    // suffisent, et la liste disait déjà « Demain » un peu plus haut.
    tiles.push(tile(
        IconName::Calendar,
        "i18n:explore.item.day",
        if view.is_later_day(stop) {
            "i18n:explore.detail.laterDay".to_string()
        } else {
            "i18n:explore.item.today".to_string()
        },
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

/// « Retard estimé de 5 min » — et ce que le voyageur doit en faire de l'heure affichée.
///
/// L'heure du tableau est déjà celle du temps réel : sans le dire, on ajoute le retard deux fois
/// et on arrive à la gare un quart d'heure trop tard.
fn delay_banner(stop: &Stop, late: i64) -> Component {
    Component::InfoBanner(
        InfoBanner::new()
            .tone(Tone::Warning)
            // « Retard estimé de 5 min », pas le « +5 min » de la rangée : ici il y a la place
            // d'une phrase, et la pastille au-dessus porte déjà la forme courte.
            .title(
                t!("explore.item.delayed.title", minutes = late)
                    .unwrap_or_else(|_| "i18n:explore.item.delayed.title".into()),
            )
            .message(
                t!("explore.item.delayed.message", time = stop.time.as_str())
                    .unwrap_or_else(|_| "i18n:explore.item.delayed.message".into()),
            ),
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

/// Le site de l'opérateur qui sert ce tableau.
const TICKET_URL: &str = "https://www.sncf-connect.com";

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
