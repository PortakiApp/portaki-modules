//! Guest home booklet card — featured && active appliances (max 5).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, EmptyState, ListItem, Pressable};
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
            .title(json!("i18n:home.card.title"))
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

pub fn device_list_item(device: &Appliance) -> Component {
    let title = if device.emoji.trim().is_empty() {
        device.name.clone()
    } else {
        format!("{} {}", device.emoji, device.name)
    };
    let mut item = ListItem::new().title(json!(title));
    if !device.location.trim().is_empty() {
        item = item.subtitle(json!(device.location.clone()));
    }
    item = item.trailing(json!("chevron-right"));
    Component::Pressable(
        Pressable::new()
            .action(json!({
                "type": "navigate",
                "to": format!("appliances/{}", device.id)
            }))
            .child(Component::ListItem(item)),
    )
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
