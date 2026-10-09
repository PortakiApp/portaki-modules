//! La carte d'accueil : le tableau en aperçu, et la carte d'avant l'arrivée.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Emphasis;
use portaki_sdk::sdui::primitives::{Card, EmptyState, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::board::BoardView;
use crate::sncf::Way;

/// Départs montrés sur la carte d'accueil, qui reste un aperçu.
const HOME_STOPS: usize = 4;

pub fn build_home_card(view: &BoardView) -> Surface {
    let mut children: Vec<Component> = vec![Component::Text(
        Text::new()
            .text(view.station.label.clone())
            .variant(TextVariant::Caption)
            .emphasis(Emphasis::Subtle),
    )];
    if view.stops.is_empty() {
        // Une carte qui n'a que le nom de sa gare laisse croire à une carte qui charge. Le
        // tableau est un contenu calculé : son vide se dit (§0.5).
        children.push(Component::EmptyState(
            EmptyState::new()
                .title("i18n:explore.detail.empty.title")
                .description("i18n:explore.detail.empty.description")
                .icon(IconName::Train),
        ));
    }
    children.extend(super::detail::last_train_banner(view));
    children.extend(
        view.stops
            .iter()
            .take(HOME_STOPS)
            .map(|stop| super::detail::departure_row(view, stop)),
    );

    Surface::new(card(children)).with_id(crate::guest::HOME_CARD)
}

/// La carte d'avant l'arrivée, délibérément petite : pas de tableau, juste le prochain train.
pub fn build_upcoming_card(view: &BoardView) -> Surface {
    let children: Vec<Component> = vec![Component::Text(
        Text::new()
            .text(upcoming_headline(view, Way::To))
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

/// « Nice-Ville → Antibes · 08:12 », ou le nom de la gare seul quand le tableau est vide.
///
/// La flèche suit le sens, sans quoi la carte d'avant l'arrivée annonce le train qu'on prendrait
/// en partant.
fn upcoming_headline(view: &BoardView, way: Way) -> String {
    // Un train supprimé ne s'annonce pas comme celui qui amène le voyageur.
    match view.stops.iter().find(|stop| !stop.cancelled) {
        Some(stop) => {
            let (from, to) = match way {
                Way::From => (&view.station.label, &stop.direction),
                Way::To => (&stop.direction, &view.station.label),
            };
            format!("{from} → {to} · {}", stop.time)
        }
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
                lat: None,
                lng: None,
            },
            stops,
            destinations: Vec::new(),
            access: None,
            today: None,
            note: String::new(),
            read_min_ago: 0,
        }
    }

    #[test]
    fn the_headline_falls_back_to_the_station_alone() {
        assert_eq!(upcoming_headline(&view(Vec::new()), Way::To), "Antibes");
        let one = Stop {
            time: "08:12".to_string(),
            date: "2026-10-04".to_string(),
            direction: "Nice-Ville".to_string(),
            headsign: None,
            mode: None,
            network: None,
            realtime: true,
            delay_min: None,
            cancelled: false,
        };
        // Avant l'arrivée, le train qui amène : la destination est le logement, pas l'inverse.
        assert_eq!(
            upcoming_headline(&view(vec![one.clone()]), Way::To),
            "Nice-Ville → Antibes · 08:12"
        );
        assert_eq!(
            upcoming_headline(&view(vec![one]), Way::From),
            "Antibes → Nice-Ville · 08:12"
        );
    }
}
