//! Guest explore item — appliance how-to detail (Portaki Guest design).

use crate::content::{
    description_plain_text, description_to_html, extract_howto_steps, Appliance, ApplianceStatus,
    AppliancesPayload,
};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::SurfaceLevel;
use portaki_sdk::sdui::primitives::{
    Button, Card, EmptyState, Eyebrow, InfoBanner, Link, ListItem, RichText, Stack, Text,
};
use portaki_sdk::sdui::surface::Surface;

#[portaki_sdk::wire(serialize)]
struct OpenHostChatPayload<'a> {
    appliance_id: &'a str,
    appliance_name: &'a str,
    context: String,
}

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
                    .title("i18n:explore.item.notFound")
                    .description("i18n:explore.item.notFound.description")
                    .icon(IconName::Plug),
            ),
        )
        .with_id(crate::ids::EXPLORE_ITEM);
    };

    Surface::new(
        Stack::new()
            .gap(14.0)
            .children(device_detail_children(device)),
    )
    .with_id(crate::ids::EXPLORE_ITEM)
}

fn device_detail_children(device: &Appliance) -> Vec<Component> {
    let mut children = vec![header_row(device)];

    let steps = extract_howto_steps(&device.description);
    if !steps.is_empty() {
        let mut howto_children: Vec<Component> = vec![Component::Eyebrow(
            Eyebrow::new().text("i18n:explore.item.howto"),
        )];
        for (index, step) in steps.iter().enumerate() {
            howto_children.push(Component::ListItem(
                ListItem::new()
                    .title((index + 1).to_string())
                    .subtitle(step.clone()),
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
                    Component::Eyebrow(Eyebrow::new().text("i18n:explore.item.howto")),
                    Component::RichText(RichText::new().content(html)),
                ]),
            ));
        }
    }

    if !device.safety_note.trim().is_empty() {
        children.push(Component::InfoBanner(
            InfoBanner::new().message(device.safety_note.clone()),
        ));
    }

    if !device.manual_url.trim().is_empty() {
        let url = device.manual_url.trim().to_string();
        let action = Action::external(url.clone());
        children.push(Component::Link(
            Link::new()
                .label("i18n:explore.item.manual")
                .href(url)
                .action(action),
        ));
    }

    let contact_action = Action::emit(
        crate::ids::OPEN_HOST_CHAT,
        Some(json_value(OpenHostChatPayload {
            appliance_id: &device.id,
            appliance_name: &device.name,
            context: description_plain_text(&device.description),
        })),
    );

    children.push(Component::Button(
        Button::new()
            .label("i18n:explore.item.contactHost")
            .variant(ButtonVariant::Outline)
            .action(contact_action),
    ));

    children
}

fn header_row(device: &Appliance) -> Component {
    let mut title_stack = Stack::new().gap(4.0).children(vec![Component::Text(
        Text::new()
            .text(device.name.clone())
            .variant(TextVariant::Display),
    )]);
    if !device.location.trim().is_empty() {
        title_stack = title_stack.child(Component::Text(
            Text::new()
                .text(device.location.clone())
                .variant(TextVariant::Caption)
                .emphasis(portaki_sdk::sdui::common::Emphasis::Subtle),
        ));
    }

    let mut header_children = Vec::new();
    if !device.emoji.trim().is_empty() {
        header_children.push(Component::Text(
            Text::new()
                .text(device.emoji.clone())
                .variant(TextVariant::Display),
        ));
    }
    header_children.push(Component::Stack(title_stack));

    Component::Stack(
        Stack::new()
            .direction(StackDirection::Horizontal)
            .gap(12.0)
            .children(header_children),
    )
}
