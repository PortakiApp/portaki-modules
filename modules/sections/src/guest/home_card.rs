//! Livret : la carte d'accueil montre le début du mot, signé de l'hôte.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::Card;

use super::block::{blocks, empty, host_byline, visible};
use super::explore_sheet::EXPLORE_SHEET;

/// Les sections que la carte montre.
const CARD_LIMIT: usize = 2;

#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    let sections = visible(&ctx)?;
    if sections.is_empty() {
        return Ok(empty(HOME_CARD));
    }
    // La signature de l'hôte vient de la plateforme : « Claire, votre hôte depuis 2019 ».
    let card = Card::new()
        .icon(IconName::Home)
        .title("i18n:home.card.title")
        .subtitle(host_byline(&ctx))
        .action(Action::open_overlay(
            OverlayPresentation::BottomSheet,
            EXPLORE_SHEET,
            OverlayArgs::new()
                .icon(IconName::Home)
                .title("i18n:home.card.title"),
        ))
        .children(blocks(&ctx, &sections, CARD_LIMIT));
    Ok(Surface::new(card).with_id(HOME_CARD))
}
