//! Guest home booklet card — « En cas de problème », section Aide.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::Card;
use portaki_sdk::sdui::surface::Surface;

/// Une carte sans contenu, et c'est le dessin (§2.22).
///
/// Elle n'énumère pas les organes et ne porte aucun ton d'alerte : un voyageur qui arrive ne doit
/// pas lire d'abord où couper le gaz. Elle dit qu'un endroit existe, et s'ouvre quand on en a
/// besoin — ce qui est aussi pourquoi aucune ligne ne la précède.
pub fn build_home_card() -> Surface {
    Surface::new(
        Card::new()
            .icon(IconName::Sliders)
            .title("i18n:home.card.title")
            .subtitle("i18n:home.card.subtitle")
            .action(Action::open_overlay(
                OverlayPresentation::Fullscreen,
                crate::guest::EXPLORE_DETAIL,
                OverlayArgs::new()
                    .icon(IconName::Sliders)
                    .title("i18n:nav.safety-shutoffs"),
            )),
    )
    .with_id(crate::guest::HOME_CARD)
}
