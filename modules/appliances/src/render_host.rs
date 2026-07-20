//! Host dashboard surface — master-detail SDUI (list + single device form).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Button, Card, EmptyState, Field, Form, List, ListItem, Page, RichTextEditor, Select, Stack,
    Text, TextInput, Toggle,
};
use portaki_sdk::sdui::surface::Surface;
use serde_json::{json, Value};

use crate::content::{Appliance, ApplianceStatus, MAX_APPLIANCES};
use crate::store;

const SELECT_NEW: &str = "__new__";
const EMIT_SURFACE_INPUT: &str = "host.surface.input";

/// Host appliances editor — device list + one editor panel + safety notice.
#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let payload = store::load_payload().unwrap_or_default();
    let selected_id = selected_id_from_input(&ctx.input);

    let list_panel = build_device_list(&payload.devices, &selected_id);
    let detail_panel = build_detail_panel(&payload.devices, &selected_id);
    let safety_card = build_safety_card(&payload.safety_notice);

    Surface::new(
        Page::new().child(
            Stack::new()
                .direction(json!("horizontal"))
                .gap(json!(24))
                .children(vec![
                    Component::Stack(
                        Stack::new()
                            .gap(json!(12))
                            .children(vec![
                                Component::Text(
                                    Text::new()
                                        .text(json!("i18n:host.list.title"))
                                        .variant(json!("caption")),
                                ),
                                list_panel,
                            ]),
                    ),
                    Component::Stack(
                        Stack::new()
                            .gap(json!(16))
                            .children(vec![detail_panel, safety_card]),
                    ),
                ]),
        ),
    )
    .with_id("main")
}

fn selected_id_from_input(input: &Value) -> String {
    input
        .get("selectedId")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("")
        .to_string()
}

fn emit_select(selected_id: &str) -> Value {
    serde_json::to_value(Action::Emit {
        event: EMIT_SURFACE_INPUT.into(),
        payload: Some(json!({ "selectedId": selected_id })),
    })
    .unwrap_or(json!({}))
}

fn build_device_list(devices: &[Appliance], selected_id: &str) -> Component {
    let mut stack_children: Vec<Component> = Vec::new();

    if devices.is_empty() {
        stack_children.push(Component::Text(
            Text::new()
                .text(json!("i18n:host.list.empty"))
                .variant(json!("caption")),
        ));
    } else {
        let items: Vec<Component> = devices
            .iter()
            .map(|device| {
                let mut item = ListItem::new()
                    .title(json!(device.name.clone()))
                    .chevron(json!(true))
                    .action(emit_select(&device.id));
                if !device.emoji.trim().is_empty() {
                    item = item.leading(json!(device.emoji.clone()));
                }
                if !device.location.trim().is_empty() {
                    item = item.subtitle(json!(device.location.clone()));
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
            .label(json!("i18n:host.list.add"))
            .action(emit_select(SELECT_NEW));
        if selected_id == SELECT_NEW {
            add = add.tone(Tone::Primary);
        }
        stack_children.push(Component::Button(add));
    }

    Component::Stack(Stack::new().gap(json!(8)).children(stack_children))
}

fn build_detail_panel(devices: &[Appliance], selected_id: &str) -> Component {
    if selected_id.is_empty() {
        return Component::EmptyState(
            EmptyState::new()
                .title(json!("i18n:host.detail.empty.title"))
                .description(json!("i18n:host.detail.empty.description"))
                .icon(json!("plug")),
        );
    }

    let is_new = selected_id == SELECT_NEW;
    let device = if is_new {
        None
    } else {
        devices.iter().find(|device| device.id == selected_id)
    };

    if !is_new && device.is_none() {
        return Component::EmptyState(
            EmptyState::new()
                .title(json!("i18n:host.detail.missing.title"))
                .description(json!("i18n:host.detail.missing.description"))
                .icon(json!("plug")),
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

    let save_action = serde_json::to_value(Action::command(
        "appliances",
        "saveAppliance",
        json!({}),
    ))
    .unwrap_or(json!({}));

    let mut form_children: Vec<Component> = vec![
        TextInput::new()
            .name(json!("id"))
            .value(json!(id))
            .into(),
        Field::new()
            .name(json!("name"))
            .label(json!("i18n:host.device.name"))
            .child(
                TextInput::new()
                    .name(json!("name"))
                    .value(json!(name)),
            )
            .into(),
        Field::new()
            .name(json!("emoji"))
            .label(json!("i18n:host.device.emoji"))
            .child(
                TextInput::new()
                    .name(json!("emoji"))
                    .value(json!(emoji)),
            )
            .into(),
        Field::new()
            .name(json!("location"))
            .label(json!("i18n:host.device.location"))
            .child(
                TextInput::new()
                    .name(json!("location"))
                    .value(json!(location)),
            )
            .into(),
        Field::new()
            .name(json!("description"))
            .label(json!("i18n:host.device.description"))
            .child(
                RichTextEditor::new()
                    .name(json!("description"))
                    .value(json!(description)),
            )
            .into(),
        Field::new()
            .name(json!("manualUrl"))
            .label(json!("i18n:host.device.manualUrl"))
            .child(
                TextInput::new()
                    .name(json!("manualUrl"))
                    .value(json!(manual_url))
                    .placeholder(json!("https://…")),
            )
            .into(),
        Field::new()
            .name(json!("featured"))
            .label(json!("i18n:host.device.featured"))
            .child(
                Toggle::new()
                    .name(json!("featured"))
                    .checked(json!(featured)),
            )
            .into(),
        Field::new()
            .name(json!("status"))
            .label(json!("i18n:host.device.status"))
            .child(
                Select::new()
                    .name(json!("status"))
                    .options(json!([
                        {"value": "active", "label": "i18n:host.device.status.active"},
                        {"value": "hidden", "label": "i18n:host.device.status.hidden"}
                    ]))
                    .value(json!(status)),
            )
            .into(),
        Button::new()
            .label(json!("i18n:host.save"))
            .action(save_action)
            .tone(Tone::Primary)
            .into(),
    ];

    if let Some(existing) = device {
        let delete_action = serde_json::to_value(Action::command(
            "appliances",
            "deleteAppliance",
            json!({ "id": existing.id }),
        ))
        .unwrap_or(json!({}));
        form_children.push(
            Button::new()
                .label(json!("i18n:host.device.delete"))
                .action(delete_action)
                .tone(Tone::Danger)
                .into(),
        );
    }

    Component::Card(
        Card::new()
            .title(json!(if is_new {
                "i18n:host.detail.new.title"
            } else {
                "i18n:host.detail.edit.title"
            }))
            .child(Form::new().children(form_children)),
    )
}

fn build_safety_card(safety_notice: &str) -> Component {
    let save_action = serde_json::to_value(Action::command(
        "appliances",
        "saveSafetyNotice",
        json!({}),
    ))
    .unwrap_or(json!({}));

    Component::Card(
        Card::new()
            .title(json!("i18n:host.safety"))
            .child(
                Form::new().children(vec![
                    Field::new()
                        .name(json!("safetyNotice"))
                        .label(json!("i18n:host.safety"))
                        .child(
                            RichTextEditor::new()
                                .name(json!("safetyNotice"))
                                .value(json!(editor_value(safety_notice))),
                        )
                        .into(),
                    Button::new()
                        .label(json!("i18n:host.safety.save"))
                        .action(save_action)
                        .into(),
                ]),
            ),
    )
}

fn editor_value(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return r#"{"type":"doc","content":[{"type":"paragraph"}]}"#.to_string();
    }
    trimmed.to_string()
}
