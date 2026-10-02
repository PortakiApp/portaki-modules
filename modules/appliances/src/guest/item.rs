//! Guest explore item — appliance how-to detail (Portaki Guest design).

use crate::content::{
    description_plain_text, description_to_html, extract_howto_steps, Appliance, ApplianceStatus,
    AppliancesPayload,
};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::{Leading, LeadingVisual, SurfaceLevel};
use portaki_sdk::sdui::primitives::{
    Button, Card, EmptyState, Eyebrow, InfoBanner, ListItem, RichText, Stack, Text,
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
        .with_id(crate::guest::EXPLORE_ITEM);
    };

    Surface::new(Stack::new().gap(14.0).children(device_detail_children(
        device,
        &payload.paper_manuals_location,
    )))
    .with_id(crate::guest::EXPLORE_ITEM)
}

/// La carte « Notices » : le lien du fabricant, et où trouver la version papier (§3.1).
///
/// Masquée quand il n'y a ni l'un ni l'autre — une carte qui annonce des notices et n'en liste
/// aucune ne dit rien. Le dépôt d'un PDF attend le lot de stockage de fichiers.
fn manuals_card(device: &Appliance, paper_location: &str) -> Option<Component> {
    let url = device.manual_url.trim();
    let paper = paper_location.trim();
    if url.is_empty() && paper.is_empty() {
        return None;
    }

    let mut rows: Vec<Component> = Vec::new();
    if !url.is_empty() {
        rows.push(Component::ListItem(
            ListItem::new()
                .title("i18n:explore.item.manual")
                .leading(Leading::Icon("file-text".into()))
                .chevron(true)
                .action(Action::external(url.to_string())),
        ));
    }
    if !paper.is_empty() {
        rows.push(Component::ListItem(
            ListItem::new()
                .title("i18n:explore.item.manual.paper")
                .subtitle(paper.to_string())
                .leading(Leading::Icon("home".into())),
        ));
    }

    Some(Component::Card(
        Card::new()
            .surface(SurfaceLevel::Elevated)
            .icon(IconName::FileText)
            .title("i18n:explore.item.manuals")
            .children(rows),
    ))
}

fn device_detail_children(device: &Appliance, paper_manuals_location: &str) -> Vec<Component> {
    let mut children = vec![header_row(device)];

    let steps = extract_howto_steps(&device.description);
    if !steps.is_empty() {
        let mut howto_children: Vec<Component> = vec![Component::Eyebrow(
            Eyebrow::new().text("i18n:explore.item.howto"),
        )];
        for (index, step) in steps.iter().enumerate() {
            // Le numéro est un repère, pas le texte de l'étape : il passe en tête de rangée et
            // l'étape reprend le titre. L'inverse se lisait « 1 » en gros, la consigne en petit.
            howto_children.push(Component::ListItem(
                ListItem::new()
                    .title(step.clone())
                    .leading(Leading::Visual(Box::new(LeadingVisual {
                        index: Some((index + 1) as u32),
                        ..LeadingVisual::default()
                    }))),
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

    if let Some(card) = manuals_card(device, paper_manuals_location) {
        children.push(card);
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
