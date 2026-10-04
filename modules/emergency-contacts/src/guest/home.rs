//! Guest home booklet card.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::Card;
use portaki_sdk::sdui::surface::Surface;

use super::body::{build_contacts_body, country_numbers};
use super::load::GuestData;

/// Les numéros du pays d'abord : en urgence, on compose avant de lire (§2.16).
///
/// En une fois, parce que `children` remplace la liste là où `child` l'allonge — un `child`
/// suivi d'un `children` perdrait silencieusement les tuiles.
fn card_body(data: &GuestData) -> Vec<Component> {
    let mut body = vec![country_numbers(&data.locale)];
    body.extend(build_contacts_body(data, false));
    body
}

pub fn build_home_card(data: &GuestData) -> Surface {
    Surface::new(
        Card::new()
            .icon(IconName::Phone)
            .title("i18n:home.card.title")
            .subtitle("i18n:home.card.subtitle")
            .action(Action::open_overlay(
                OverlayPresentation::BottomSheet,
                crate::guest::EXPLORE_DETAIL,
                OverlayArgs::new()
                    .icon(IconName::Phone)
                    .title("i18n:home.card.title"),
            ))
            .children(card_body(data)),
    )
    .with_id(crate::guest::HOME_CARD)
}
