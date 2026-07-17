//! Guest home booklet card — featured && active appliances (max 5).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{Card, EmptyState, ListItem};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

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
            .icon(json!("plug"))
            .title(json!("i18n:nav.appliances"))
            .action(json!({
                "type": "navigate",
                "to": "appliances"
            }))
            .children(if children.is_empty() {
                vec![Component::EmptyState(
                    EmptyState::new()
                        .title(json!("i18n:home.card.featured.empty.title"))
                        .description(json!("i18n:home.card.featured.empty.description"))
                        .icon(json!("plug")),
                )]
            } else {
                children
            }),
    )
    .with_id("home.card")
}

/// List row matching Portaki Guest design: emoji leading, name, location, chevron.
pub fn device_list_item(device: &Appliance) -> Component {
    let action = serde_json::to_value(Action::Navigate {
        to: format!("appliances/{}", device.id),
        params: None,
    })
    .unwrap_or(json!({}));

    let mut item = ListItem::new()
        .title(json!(device.name.clone()))
        .chevron(json!(true))
        .action(action);

    if !device.emoji.trim().is_empty() {
        item = item.leading(json!(device.emoji.clone()));
    }
    if !device.location.trim().is_empty() {
        item = item.subtitle(json!(device.location.clone()));
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
                .title(json!("i18n:explore.detail.empty.title"))
                .description(json!("i18n:explore.detail.empty.description"))
                .icon(json!("plug")),
        ));
    }
    children
}
