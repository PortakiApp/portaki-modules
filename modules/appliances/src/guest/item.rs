//! Guest explore item — appliance how-to detail (Portaki Guest design).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::SurfaceLevel;
use portaki_sdk::sdui::primitives::{
    Button, Card, EmptyState, Eyebrow, InfoBanner, Link, ListItem, RichText, Stack, Text,
};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use crate::content::{
    description_plain_text, description_to_html, extract_howto_steps, Appliance, ApplianceStatus,
    AppliancesPayload,
};

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
            .gap(json!(14))
            .children(device_detail_children(device)),
    )
    .with_id("explore.item")
}

fn device_detail_children(device: &Appliance) -> Vec<Component> {
    let mut children = vec![header_row(device)];

    let steps = extract_howto_steps(&device.description);
    if !steps.is_empty() {
        let mut howto_children: Vec<Component> = vec![Component::Eyebrow(
            Eyebrow::new().text(json!("i18n:explore.item.howto")),
        )];
        for (index, step) in steps.iter().enumerate() {
            howto_children.push(Component::ListItem(
                ListItem::new()
                    .title(json!((index + 1).to_string()))
                    .subtitle(json!(step.clone())),
            ));
        }
        children.push(Component::Card(
            Card::new()
                .surface(SurfaceLevel::Elevated)
                .children(howto_children),
        ));
    } else {
        let html = description_to_html(&device.description);
        if !html.trim().is_empty() {
            children.push(Component::Card(
                Card::new().surface(SurfaceLevel::Elevated).children(vec![
                    Component::Eyebrow(Eyebrow::new().text(json!("i18n:explore.item.howto"))),
                    Component::RichText(RichText::new().content(json!(html))),
                ]),
            ));
        }
    }

    if !device.safety_note.trim().is_empty() {
        children.push(Component::InfoBanner(
            InfoBanner::new().message(json!(device.safety_note.clone())),
        ));
    }

    if !device.manual_url.trim().is_empty() {
        let url = device.manual_url.trim().to_string();
        let action =
            serde_json::to_value(Action::External { url: url.clone() }).unwrap_or(json!({}));
        children.push(Component::Link(
            Link::new()
                .label(json!("i18n:explore.item.manual"))
                .href(json!(url))
                .action(action),
        ));
    }

    let contact_action = serde_json::to_value(Action::Emit {
        event: "openHostChat".into(),
        payload: Some(json!({
            "applianceId": device.id,
            "applianceName": device.name,
            "context": description_plain_text(&device.description),
        })),
    })
    .unwrap_or(json!({}));

    children.push(Component::Button(
        Button::new()
            .label(json!("i18n:explore.item.contactHost"))
            .variant(json!("outline"))
            .action(contact_action),
    ));

    children
}

fn header_row(device: &Appliance) -> Component {
    let mut title_stack = Stack::new().gap(json!(4)).children(vec![Component::Text(
        Text::new()
            .text(json!(device.name.clone()))
            .variant(json!("title")),
    )]);
    if !device.location.trim().is_empty() {
        title_stack = title_stack.child(Component::Text(
            Text::new()
                .text(json!(device.location.clone()))
                .variant(json!("caption"))
                .emphasis(portaki_sdk::sdui::common::Emphasis::Subtle),
        ));
    }

    let mut header_children = Vec::new();
    if !device.emoji.trim().is_empty() {
        header_children.push(Component::Text(
            Text::new()
                .text(json!(device.emoji.clone()))
                .variant(json!("display")),
        ));
    }
    header_children.push(Component::Stack(title_stack));

    Component::Stack(
        Stack::new()
            .direction(json!("horizontal"))
            .gap(json!(12))
            .children(header_children),
    )
}
