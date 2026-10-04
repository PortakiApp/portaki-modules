//! Guest home booklet card.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::Card;
use portaki_sdk::sdui::surface::Surface;

use super::body::{build_hours_body, stay_tiles};
use super::load::GuestData;

/// Les tuiles du séjour d'abord, puis les horaires (§2.6).
///
/// En une fois : `children` remplace la liste là où `child` l'allonge.
fn card_body(data: &GuestData) -> Vec<Component> {
    let mut body: Vec<Component> = stay_tiles(data).into_iter().collect();
    body.extend(build_hours_body(data, false));
    body
}

pub fn build_home_card(data: &GuestData) -> Surface {
    Surface::new(
        Card::new()
            .icon(IconName::Clock)
            .title("i18n:home.card.title")
            .subtitle("i18n:home.card.subtitle")
            .action(Action::open_overlay(
                OverlayPresentation::BottomSheet,
                crate::guest::EXPLORE_DETAIL,
                OverlayArgs::new()
                    .icon(IconName::Clock)
                    .title("i18n:home.card.title"),
            ))
            .children(card_body(data)),
    )
    .with_id(crate::guest::HOME_CARD)
}
