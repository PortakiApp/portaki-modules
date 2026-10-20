//! La carte d'accueil : les appareils mis en avant, en tuiles (§2.4).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::ListItemLayout;
use portaki_sdk::sdui::primitives::{Button, Card, Grid};

use super::explore_detail::EXPLORE_DETAIL;
use super::row::{device_list_item, no_appliances};
use crate::domain::{Appliance, Guide};

#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    let guide = Guide::load(&ctx)?;
    if guide.is_empty() {
        return Ok(no_appliances(HOME_CARD));
    }
    // Des tuiles dans une grille : quatre vignettes se balaient d'un coup d'œil (§2.4).
    let tiles: Vec<Component> = guide.featured().into_iter().map(device_tile).collect();
    let total = guide.devices.len();
    let card = Card::new()
        .icon(IconName::Plug)
        .title("i18n:nav.appliances")
        .subtitle(t!("home.card.subtitle", count = total).unwrap_or_default())
        .action(open_list())
        .children(vec![
            Grid::new()
                .minColumnWidth(140.0)
                .plain(true)
                .children(tiles)
                .into(),
            see_all(total),
        ]);
    Ok(Surface::new(card).with_id(HOME_CARD))
}

fn open_list() -> Action {
    Action::open_overlay(
        OverlayPresentation::Fullscreen,
        EXPLORE_DETAIL,
        OverlayArgs::new()
            .icon(IconName::Plug)
            .title("i18n:nav.appliances"),
    )
}

/// « Voir les N appareils » : le compte de la liste, pas celui des tuiles. Sans service de
/// traduction, une clé sans nombre que le livret résout lui-même — jamais un bouton vide.
fn see_all(count: usize) -> Component {
    let label = t!("home.card.seeAll", count = count)
        .unwrap_or_else(|_| "i18n:home.card.seeAllPlain".to_string());
    Button::new()
        .label(label)
        .variant(ButtonVariant::Outline)
        .action(open_list())
        .into()
}

/// Un appareil en tuile : même contenu qu'une ligne, lu d'un coup.
fn device_tile(device: &Appliance) -> Component {
    let Component::ListItem(item) = device_list_item(device) else {
        unreachable!("device_list_item rend toujours un ListItem")
    };
    Component::ListItem(item.layout(ListItemLayout::Tile).chevron(false))
}
