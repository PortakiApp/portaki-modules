//! Guest home booklet card.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::Card;
use portaki_sdk::sdui::surface::Surface;

use super::body::build_contacts_body;
use super::load::GuestData;

pub fn build_home_card(data: &GuestData) -> Surface {
    Surface::new(
        Card::new()
            .icon(IconName::Phone)
            .title("i18n:home.card.title")
            .action(Action::open_overlay(
                OverlayPresentation::BottomSheet,
                crate::ids::EXPLORE_DETAIL,
                OverlayArgs::new()
                    .icon(IconName::Phone)
                    .title("i18n:home.card.title"),
            ))
            .children(build_contacts_body(data, false)),
    )
    .with_id(crate::ids::HOME_CARD)
}
