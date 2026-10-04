//! La carte d'accueil : le tableau en aperçu, et la carte d'avant l'arrivée.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Emphasis;
use portaki_sdk::sdui::primitives::{Card, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::board::BoardView;

/// Départs montrés sur la carte d'accueil, qui reste un aperçu.
const HOME_STOPS: usize = 4;

pub fn build_home_card(view: &BoardView) -> Surface {
    let mut children: Vec<Component> = vec![Component::Text(
        Text::new()
            .text(view.station.label.clone())
            .variant(TextVariant::Caption)
            .emphasis(Emphasis::Subtle),
    )];
    children.extend(
        view.stops
            .iter()
            .take(HOME_STOPS)
            .map(|stop| super::detail::departure_row(view, stop)),
    );

    Surface::new(card(children)).with_id(crate::guest::HOME_CARD)
}

/// La carte d'avant l'arrivée, délibérément petite : pas de tableau, juste le prochain départ.
pub fn build_upcoming_card(view: &BoardView) -> Surface {
    let children: Vec<Component> = vec![Component::Text(
        Text::new()
            .text(upcoming_headline(view))
            .variant(TextVariant::Caption)
            .emphasis(Emphasis::Subtle),
    )];

    Surface::new(card(children)).with_id(crate::guest::UPCOMING_CARD)
}

fn card(children: Vec<Component>) -> Card {
    Card::new()
        .icon(IconName::Train)
        .title("i18n:home.card.title")
        .action(Action::open_overlay(
            OverlayPresentation::Fullscreen,
            crate::guest::EXPLORE_DETAIL,
            OverlayArgs::new()
                .icon(IconName::Train)
                .title("i18n:home.card.title"),
        ))
        .children(children)
}

/// « Antibes → Nice-Ville · 08:12 », ou le nom de la gare seul quand le tableau est vide.
fn upcoming_headline(view: &BoardView) -> String {
    match view.stops.first() {
        Some(stop) => format!(
            "{} → {} · {}",
            view.station.label, stop.direction, stop.time
        ),
        None => view.station.label.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sncf::{Station, Stop};

    fn view(stops: Vec<Stop>) -> BoardView {
        BoardView {
            station: Station {
                id: "stop_area:SNCF:87756056".to_string(),
                label: "Antibes".to_string(),
            },
            stops,
            destinations: Vec::new(),
            today: None,
            note: String::new(),
        }
    }

    #[test]
    fn the_headline_falls_back_to_the_station_alone() {
        assert_eq!(upcoming_headline(&view(Vec::new())), "Antibes");
        let one = Stop {
            time: "08:12".to_string(),
            date: "2026-10-04".to_string(),
            direction: "Nice-Ville".to_string(),
            headsign: None,
            mode: None,
            network: None,
            realtime: true,
        };
        assert_eq!(
            upcoming_headline(&view(vec![one])),
            "Antibes → Nice-Ville · 08:12"
        );
    }
}
