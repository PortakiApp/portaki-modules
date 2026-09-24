//! Guest home booklet card — featured && active appliances (max 5).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{Card, EmptyState, ListItem};
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
                crate::ids::EXPLORE_DETAIL,
                OverlayArgs::new()
                    .icon(IconName::Plug)
                    .title("i18n:nav.appliances"),
            ))
            .children(if children.is_empty() {
                vec![Component::EmptyState(
                    EmptyState::new()
                        .title("i18n:home.card.featured.empty.title")
                        .description("i18n:home.card.featured.empty.description")
                        .icon(IconName::Plug),
                )]
            } else {
                children
            }),
    )
    .with_id(crate::ids::HOME_CARD)
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
        item = item.leading(device.emoji.clone());
    }
    if !device.location.trim().is_empty() {
        item = item.subtitle(device.location.clone());
    }

    Component::ListItem(item)
}

pub fn devices_list(payload: &AppliancesPayload) -> Vec<Component> {
    let mut children: Vec<Component> = payload
        .guest_devices()
        .into_iter()
        .map(device_list_item)
        .collect();
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
