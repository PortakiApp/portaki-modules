//! Guest explore item — appliance how-to detail with TipTap HTML.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{EmptyState, InfoBanner, Link, RichText, Stack, Text};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use crate::content::{description_to_html, Appliance, ApplianceStatus, AppliancesPayload};

pub fn build_item_detail(payload: &AppliancesPayload, device_id: Option<&str>) -> Surface {
    let device = device_id.and_then(|id| {
        payload
            .guest_devices()
            .into_iter()
            .find(|d| d.id == id)
            .or_else(|| {
                payload
                    .find_device(id)
                    .filter(|d| d.status == ApplianceStatus::Active)
            })
    });

    let Some(device) = device else {
        return Surface::new(
            Stack::new().child(
                EmptyState::new()
                    .title(json!("i18n:explore.item.notFound"))
                    .description(json!("i18n:explore.item.notFound.description"))
                    .icon(json!("plug")),
            ),
        )
        .with_id("explore.item");
    };

    Surface::new(
        Stack::new()
            .gap(json!(12))
            .children(device_detail_children(device)),
    )
    .with_id("explore.item")
}

fn device_detail_children(device: &Appliance) -> Vec<Component> {
    let title = if device.emoji.trim().is_empty() {
        device.name.clone()
    } else {
        format!("{} {}", device.emoji, device.name)
    };
    let mut children = vec![Component::Text(
        Text::new().text(json!(title)).variant(json!("title")),
    )];
    if !device.location.trim().is_empty() {
        children.push(Component::Text(
            Text::new()
                .text(json!("i18n:explore.item.location"))
                .variant(json!("caption")),
        ));
        children.push(Component::Text(
            Text::new()
                .text(json!(device.location.clone()))
                .variant(json!("body")),
        ));
    }

    if !device.safety_note.trim().is_empty() {
        children.push(Component::InfoBanner(
            InfoBanner::new()
                .title(json!("i18n:explore.item.safety"))
                .message(json!(device.safety_note.clone())),
        ));
    }

    let html = description_to_html(&device.description);
    if !html.trim().is_empty() {
        children.push(Component::Text(
            Text::new()
                .text(json!("i18n:explore.item.howto"))
                .variant(json!("caption")),
        ));
        children.push(Component::RichText(
            RichText::new().content(json!(html)),
        ));
    }

    if !device.manual_url.trim().is_empty() {
        let url = device.manual_url.trim().to_string();
        let action = serde_json::to_value(Action::External { url: url.clone() })
            .unwrap_or(json!({}));
        children.push(Component::Link(
            Link::new()
                .label(json!("i18n:explore.item.manual"))
                .href(json!(url))
                .action(action),
        ));
    }

    children
}
