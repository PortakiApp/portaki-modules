//! Host dashboard surface — master-detail SDUI (list + single device form).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Accordion, Button, Card, EmptyState, Field, Form, List, ListItem, Page, RichTextEditor, Select,
    Stack, Text, TextInput, Toggle,
};
use portaki_sdk::sdui::surface::Surface;

use crate::content::{description_plain_text, Appliance, ApplianceStatus, MAX_APPLIANCES};
use crate::store;

const SELECT_NEW: &str = "__new__";

/// Host appliances editor — safety accordion (col-12) + list (col-3) / detail (col-9).
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::AppliancesEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Plug
)]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let payload = store::load_payload_for(&ctx.locale, &ctx.property.locale).unwrap_or_default();
    let selected_id = ctx.input_str("selectedId").unwrap_or("").to_string();

    let safety = build_safety_accordion(&payload.safety_notice);
    let list_card = build_list_card(&payload.devices, &selected_id);
    let detail_panel = build_detail_panel(&payload.devices, &selected_id);

    Surface::new(Page::new().children(vec![
            safety,
            Component::Stack(
                Stack::new()
                    .direction(StackDirection::Horizontal)
                    .gap(24.0)
                    .children(vec![list_card, detail_panel]),
            ),
        ]))
    .with_id(MAIN)
}

#[portaki_sdk::wire(serialize)]
struct SurfaceInputSelectedId<'a> {
    selected_id: &'a str,
}

fn emit_select(selected_id: &str) -> Action {
    Action::emit(
        contracts::shell::SURFACE_INPUT,
        Some(json_value(SurfaceInputSelectedId { selected_id })),
    )
}

fn build_list_card(devices: &[Appliance], selected_id: &str) -> Component {
    let mut stack_children: Vec<Component> = Vec::new();

    if devices.is_empty() {
        stack_children.push(Component::Text(
            Text::new()
                .text("i18n:host.list.empty")
                .variant(TextVariant::Caption),
        ));
    } else {
        let items: Vec<Component> = devices
            .iter()
            .map(|device| {
                let mut item = ListItem::new()
                    .title(device.name.clone())
                    .chevron(true)
                    .action(emit_select(&device.id));
                if !device.emoji.trim().is_empty() {
                    item = item.leading(device.emoji.clone());
                }
                if !device.location.trim().is_empty() {
                    item = item.subtitle(device.location.clone());
                }
                if selected_id == device.id {
                    item = item.tone(Tone::Primary);
                }
                Component::ListItem(item)
            })
            .collect();
        stack_children.push(Component::List(List::new().children(items)));
    }

    if devices.len() < MAX_APPLIANCES {
        let mut add = Button::new()
            .label("i18n:host.list.add")
            .action(emit_select(SELECT_NEW));
        if selected_id == SELECT_NEW {
            add = add.tone(Tone::Primary);
        }
        stack_children.push(Component::Button(add));
    }

    Component::Card(
        Card::new()
            .title("i18n:host.list.title")
            .child(Stack::new().gap(10.0).children(stack_children)),
    )
}

fn build_detail_panel(devices: &[Appliance], selected_id: &str) -> Component {
    if selected_id.is_empty() {
        return Component::Card(
            Card::new().title("i18n:host.detail.card.title").child(
                EmptyState::new()
                    .title("i18n:host.detail.empty.title")
                    .description("i18n:host.detail.empty.description")
                    .icon(IconName::Plug),
            ),
        );
    }

    let is_new = selected_id == SELECT_NEW;
    let device = if is_new {
        None
    } else {
        devices.iter().find(|device| device.id == selected_id)
    };

    if !is_new && device.is_none() {
        return Component::Card(
            Card::new().title("i18n:host.detail.card.title").child(
                EmptyState::new()
                    .title("i18n:host.detail.missing.title")
                    .description("i18n:host.detail.missing.description")
                    .icon(IconName::Plug),
            ),
        );
    }

    let id = device.map(|d| d.id.as_str()).unwrap_or("");
    let name = device.map(|d| d.name.as_str()).unwrap_or("");
    let emoji = device.map(|d| d.emoji.as_str()).unwrap_or("");
    let location = device.map(|d| d.location.as_str()).unwrap_or("");
    let manual_url = device.map(|d| d.manual_url.as_str()).unwrap_or("");
    let description = device
        .map(|d| editor_value(&d.description))
        .unwrap_or_else(|| editor_value(""));
    let featured = device.map(|d| d.featured).unwrap_or(false);
    let status = device
        .map(|d| match d.status {
            ApplianceStatus::Hidden => "hidden",
            ApplianceStatus::Active => "active",
        })
        .unwrap_or("active");

    let save_action = crate::ids::module_id().command_empty(crate::commands::SAVE_APPLIANCE);

    let mut form_children: Vec<Component> = vec![
        TextInput::new().name("id").value(id).into(),
        Field::new()
            .name("name")
            .label("i18n:host.device.name")
            .child(TextInput::new().name("name").value(name))
            .into(),
        Field::new()
            .name("emoji")
            .label("i18n:host.device.emoji")
            .child(TextInput::new().name("emoji").value(emoji))
            .into(),
        Field::new()
            .name("location")
            .label("i18n:host.device.location")
            .child(TextInput::new().name("location").value(location))
            .into(),
        Field::new()
            .name("description")
            .label("i18n:host.device.description")
            .child(RichTextEditor::new().name("description").value(description))
            .into(),
        Field::new()
            .name("manualUrl")
            .label("i18n:host.device.manualUrl")
            .child(
                TextInput::new()
                    .name("manualUrl")
                    .value(manual_url)
                    .placeholder("https://…"),
            )
            .into(),
        Field::new()
            .name("featured")
            .label("i18n:host.device.featured")
            .child(Toggle::new().name("featured").checked(featured))
            .into(),
        Field::new()
            .name("status")
            .label("i18n:host.device.status")
            .child(
                Select::new()
                    .name("status")
                    .options(vec![
                        ChoiceOption::new("active", "i18n:host.device.status.active"),
                        ChoiceOption::new("hidden", "i18n:host.device.status.hidden"),
                    ])
                    .value(status),
            )
            .into(),
        Button::new()
            .label("i18n:host.save")
            .action(save_action)
            .tone(Tone::Primary)
            .into(),
    ];

    if let Some(existing) = device {
        let delete_action = crate::ids::module_id().command(
            crate::commands::DELETE_APPLIANCE,
            crate::commands::DeleteApplianceArgs {
                id: existing.id.clone(),
            },
        );
        form_children.push(
            Button::new()
                .label("i18n:host.device.delete")
                .action(delete_action)
                .tone(Tone::Danger)
                .into(),
        );
    }

    Component::Card(
        Card::new()
            .title(if is_new {
                "i18n:host.detail.new.title"
            } else {
                "i18n:host.detail.edit.title"
            })
            .child(Form::new().children(form_children)),
    )
}

fn build_safety_accordion(safety_notice: &str) -> Component {
    let save_action = crate::ids::module_id().command_empty(crate::commands::SAVE_SAFETY_NOTICE);
    let has_value = !description_plain_text(safety_notice).trim().is_empty();
    // Shell Accordion: `:collapsed` → closed by default; otherwise open.
    let accordion_id = if has_value {
        "host.safety:collapsed"
    } else {
        "host.safety:expanded"
    };

    Component::Accordion(
        Accordion::new()
            .id(accordion_id)
            .child(
                Card::new()
                    .title("i18n:host.safety")
                    .child(Form::new().children(vec![
                    Text::new()
                        .text("i18n:host.safety.hint")
                        .variant(TextVariant::Caption)
                        .into(),
                    Field::new()
                        .name("safetyNotice")
                        .child(
                            RichTextEditor::new()
                                .name("safetyNotice")
                                .value(editor_value(safety_notice)),
                        )
                        .into(),
                    Button::new()
                        .label("i18n:host.safety.save")
                        .action(save_action)
                        .tone(Tone::Primary)
                        .into(),
                ])),
            ),
    )
}

fn editor_value(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return r#"{"type":"doc","content":[{"type":"paragraph"}]}"#.to_string();
    }
    // Already TipTap JSON — keep as-is; plain text is wrapped by the shell parser.
    trimmed.to_string()
}
