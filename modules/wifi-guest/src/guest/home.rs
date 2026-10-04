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
            // La note de l'hôte — « Fibre · couvre la terrasse » — dit en une ligne ce que le
            // voyageur veut savoir avant d'ouvrir. Pas de note, pas de sous-titre inventé.
            .subtitle(
                data.config
                    .hint
                    .as_ref()
                    .map(|hint| hint.get(&data.locale).trim().to_string())
                    .filter(|note| !note.is_empty())
                    .unwrap_or_default(),
            )
            .action(Action::open_overlay(
                OverlayPresentation::BottomSheet,
                crate::guest::EXPLORE_DETAIL,
                OverlayArgs::new()
                    .icon(IconName::Wifi)
                    .title("i18n:nav.wifi-guest"),
            ))
            .children(build_wifi_body(data, false)),
    )
    .with_id(crate::guest::HOME_CARD)
}
