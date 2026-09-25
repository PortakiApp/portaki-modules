//! Guest home booklet card.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::Card;
use portaki_sdk::sdui::surface::Surface;

use super::body::build_hours_body;
use super::load::GuestData;

pub fn build_home_card(data: &GuestData) -> Surface {
    Surface::new(
        Card::new()
            .icon(IconName::Clock)
            .title("i18n:home.card.title")
            .action(Action::open_overlay(
                OverlayPresentation::BottomSheet,
                crate::guest::EXPLORE_DETAIL,
                OverlayArgs::new()
                    .icon(IconName::Clock)
                    .title("i18n:home.card.title"),
            ))
            .children(build_hours_body(data, false)),
    )
    .with_id(crate::guest::HOME_CARD)
}
