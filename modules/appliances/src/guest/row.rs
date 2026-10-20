//! Ce que les surfaces du livret partagent : la ligne d'un appareil, et l'écran sans appareil.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{Leading, LeadingVisual};
use portaki_sdk::sdui::primitives::{EmptyState, ListItem};

use crate::domain::Appliance;

/// Aucun appareil publié.
pub(crate) fn no_appliances(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:home.card.empty.title")
            .description("i18n:home.card.empty.description")
            .icon(IconName::Plug),
    )
    .with_id(surface_id)
}

/// Une ligne de la liste : l'emoji, le nom, la pièce ; elle ouvre la fiche.
pub(crate) fn device_list_item(device: &Appliance) -> Component {
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
