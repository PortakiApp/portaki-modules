//! Livret : la carte d'accueil en tuiles, la liste par pièce, la fiche d'un appareil.

mod explore_detail;
mod explore_item;
mod home_card;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{Leading, LeadingVisual};
use portaki_sdk::sdui::primitives::{EmptyState, ListItem};

pub use explore_detail::render_explore_detail;
pub use explore_item::render_explore_item;
pub use home_card::render_home_card;

use crate::domain::Appliance;

/// Aucun appareil publié.
fn no_appliances(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:home.card.empty.title")
            .description("i18n:home.card.empty.description")
            .icon(IconName::Plug),
    )
    .with_id(surface_id)
}

/// Une ligne de la liste : l'emoji, le nom, la pièce ; elle ouvre la fiche.
fn device_list_item(device: &Appliance) -> Component {
    let action = Action::navigate(
        NavigateTarget::path(format!("appliances/{}", device.id)),
        None,
    );
    let mut item = ListItem::new()
        .title(device.name.clone())
        .chevron(true)
        .action(action);
    if !device.emoji.is_empty() {
        // `emoji`, pas la forme nue `Leading::Icon` : nue, le livret la prend pour une icône et la
        // pose sur la pastille teintée ; nommée, il dessine la pastille blanche de la maquette.
        item = item.leading(Leading::Visual(Box::new(LeadingVisual {
            emoji: Some(device.emoji.clone()),
            ..LeadingVisual::default()
        })));
    }
    if !device.location.is_empty() {
        item = item.subtitle(device.location.clone());
    }
    Component::ListItem(item)
}
