//! Guest home booklet card — featured && active appliances (§2.4).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::Leading;
use portaki_sdk::sdui::primitives::{Button, Card, EmptyState, Eyebrow, ListItem};
use portaki_sdk::sdui::surface::Surface;

use crate::content::{Appliance, AppliancesPayload};

/// Home card: featured active devices only. Card → list path; row → detail path.
pub fn build_home_card(payload: &AppliancesPayload) -> Surface {
    let children: Vec<Component> = payload
        .featured_guest_devices()
        .into_iter()
        .map(device_list_item)
        .collect();

    Surface::new(
        Card::new()
            .icon(IconName::Plug)
            .title("i18n:nav.appliances")
            .action(Action::open_overlay(
                OverlayPresentation::Fullscreen,
                crate::guest::EXPLORE_DETAIL,
                OverlayArgs::new()
                    .icon(IconName::Plug)
                    .title("i18n:nav.appliances"),
            ))
            .children(if children.is_empty() {
                // Aucun appareil mis en avant : un état vide imbriqué, pas une carte vide — et le
                // bouton reste, puisque la liste, elle, n'est pas vide (§2.4).
                vec![
                    Component::EmptyState(
                        EmptyState::new()
                            .title("i18n:home.card.featured.empty.title")
                            .description("i18n:home.card.featured.empty.description")
                            .icon(IconName::Plug),
                    ),
                    see_all_button(payload),
                ]
            } else {
                let mut rows = children;
                rows.push(see_all_button(payload));
                rows
            }),
    )
    .with_id(crate::guest::HOME_CARD)
}

/// « Voir les N appareils » (§2.4) — le compte est celui de la liste, pas celui des tuiles.
///
/// Sans lui, la carte n'avouait pas qu'il y avait autre chose : son action ouvrait bien la liste,
/// mais rien ne le disait, et quatre tuiles sur trente ressemblaient à trente.
fn see_all_button(payload: &AppliancesPayload) -> Component {
    // Le compte est résolu ici et non par le livret : une clé `i18n:` traverse telle quelle, et le
    // livret n'a pas de quoi y glisser un nombre.
    let count = payload.guest_devices().len();
    // Deux filets, parce qu'un libellé de bouton ne peut ni être vide ni afficher un identifiant.
    // Le service de traduction de l'hôte sait glisser le nombre ; s'il ne répond pas, on retombe sur
    // une clé `i18n:` sans nombre, que le livret résout lui-même côté client. Un `unwrap_or_default`
    // aurait rendu un bouton sans texte.
    let label = t!("home.card.seeAll", count = count)
        .unwrap_or_else(|_| "i18n:home.card.seeAllPlain".to_string());
    Component::Button(
        Button::new()
            .label(label)
            .variant(ButtonVariant::Outline)
            .action(Action::open_overlay(
                OverlayPresentation::Fullscreen,
                crate::guest::EXPLORE_DETAIL,
                OverlayArgs::new()
                    .icon(IconName::Plug)
                    .title("i18n:nav.appliances"),
            )),
    )
}

/// List row matching Portaki Guest design: emoji leading, name, location, chevron.
pub fn device_list_item(device: &Appliance) -> Component {
    let action = Action::navigate(
        NavigateTarget::path(format!("appliances/{}", device.id)),
        None,
    );

    let mut item = ListItem::new()
        .title(device.name.clone())
        .chevron(true)
        .action(action);

    if !device.emoji.trim().is_empty() {
        item = item.leading(Leading::Icon(device.emoji.clone()));
    }
    if !device.location.trim().is_empty() {
        item = item.subtitle(device.location.clone());
    }

    Component::ListItem(item)
}

pub fn devices_list(payload: &AppliancesPayload) -> Vec<Component> {
    let mut children: Vec<Component> = Vec::new();
    let rooms = payload.guest_devices_by_room();
    // Un seul groupe : pas de titre. Il n'apprendrait rien qu'on ne lise déjà sur chaque ligne, où
    // la pièce est en sous-titre, et un logement d'une seule pièce n'a pas de plan à annoncer.
    let titled = rooms.len() > 1;
    for (room, devices) in rooms {
        if titled {
            children.push(Component::Eyebrow(Eyebrow::new().text(
                room.unwrap_or_else(|| "i18n:explore.detail.room.other".to_string()),
            )));
        }
        children.extend(devices.into_iter().map(device_list_item));
    }
    if children.is_empty() {
        children.push(Component::EmptyState(
            EmptyState::new()
                .title("i18n:explore.detail.empty.title")
                .description("i18n:explore.detail.empty.description")
                .icon(IconName::Plug),
        ));
    }
    children
}
