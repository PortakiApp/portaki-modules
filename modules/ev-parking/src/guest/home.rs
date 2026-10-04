//! Guest home booklet card.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::Card;
use portaki_sdk::sdui::surface::Surface;

use super::body::build_ev_parking_body;
use super::load::GuestData;

pub fn build_home_card(data: &GuestData) -> Surface {
    Surface::new(
        Card::new()
            .icon(IconName::Zap)
            .title("i18n:nav.ev-parking")
            // L'emplacement — « Place P2 n° 14 · niveau −1 » — est ce qu'on relit en descendant
            // au parking, pas ce qu'on va chercher dans une sous-page.
            .subtitle(data.config.spot_label.get(&data.locale).trim())
            .action(Action::open_overlay(
                OverlayPresentation::BottomSheet,
                crate::guest::EXPLORE_DETAIL,
                OverlayArgs::new()
                    .icon(IconName::Zap)
                    .title("i18n:nav.ev-parking"),
            ))
            .children(build_ev_parking_body(data, false)),
    )
    .with_id(crate::guest::HOME_CARD)
}
