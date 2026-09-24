//! Guest home booklet card.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::Card;
use portaki_sdk::sdui::surface::Surface;

use super::body::build_access_glance;
use super::load::GuestData;

pub fn build_home_card(data: &GuestData) -> Surface {
    // Prefer nav.* — shell always ships `nav.access-guide`, so a colliding
    // `home.card.title` from another module bundle cannot overwrite this label.
    Surface::new(
        Card::new()
            .icon(IconName::Car)
            .title("i18n:nav.access-guide")
            .action(Action::open_overlay(
                OverlayPresentation::Fullscreen,
                crate::ids::EXPLORE_DETAIL,
                OverlayArgs::new()
                    .icon(IconName::Car)
                    .title("i18n:nav.access-guide"),
            ))
            .children(build_access_glance(data)),
    )
    .with_id(crate::ids::HOME_CARD)
}
