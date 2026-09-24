//! Guest home booklet card.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::Card;
use portaki_sdk::sdui::surface::Surface;

use super::body::build_wifi_body;
use super::load::GuestData;

pub fn build_home_card(data: &GuestData) -> Surface {
    Surface::new(
        Card::new()
            .icon(IconName::Wifi)
            .title("i18n:nav.wifi-guest")
            .action(Action::open_overlay(
                OverlayPresentation::BottomSheet,
                crate::ids::EXPLORE_DETAIL,
                OverlayArgs::new()
                    .icon(IconName::Wifi)
                    .title("i18n:nav.wifi-guest"),
            ))
            .children(build_wifi_body(data, false)),
    )
    .with_id(crate::ids::HOME_CARD)
}
