//! Host dashboard surfaces.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{Button, Field, Form, Page, Select, Text, TextArea, TextInput};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use crate::config::{load_config, AccessStep};

const STEP_SLOTS: usize = 6;

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let _ = ctx;
    let config = load_config().unwrap_or_default();
    let steps = config.parse_steps();

    let submit_args = json!({
        "steps": steps_to_submit(&steps),
        "parking_map_url": config.parking_map_url,
        "arrival_video_url": config.arrival_video_url,
        "global_note": config.global_note,
        "address": config.address,
        "gate_code": config.gate_code,
        "keybox_code": config.keybox_code,
        "parking_info": config.parking_info,
    });
    let save_action =
        serde_json::to_value(Action::command("access-guide", "updateConfig", submit_args))
            .unwrap_or(json!({}));

    let mut form_children: Vec<Component> = vec![
        Field::new()
            .name(json!("address"))
            .label(json!("i18n:host.address.label"))
            .child(
                TextInput::new()
                    .name(json!("address"))
                    .value(json!(config.address)),
            )
            .into(),
        Field::new()
            .name(json!("gate_code"))
            .label(json!("i18n:host.gate.label"))
            .child(
                TextInput::new()
                    .name(json!("gate_code"))
                    .value(json!(config.gate_code)),
            )
            .into(),
        Field::new()
            .name(json!("keybox_code"))
            .label(json!("i18n:host.keybox.label"))
            .child(
                TextInput::new()
                    .name(json!("keybox_code"))
                    .value(json!(config.keybox_code)),
            )
            .into(),
        Field::new()
            .name(json!("parking_info"))
            .label(json!("i18n:host.parking.label"))
            .child(
                TextInput::new()
                    .name(json!("parking_info"))
                    .value(json!(config.parking_info)),
            )
            .into(),
        Field::new()
            .name(json!("parking_map_url"))
            .label(json!("i18n:host.map.label"))
            .child(
                TextInput::new()
                    .name(json!("parking_map_url"))
                    .value(json!(config.parking_map_url)),
            )
            .into(),
        Field::new()
            .name(json!("arrival_video_url"))
            .label(json!("i18n:host.video.label"))
            .child(
                TextInput::new()
                    .name(json!("arrival_video_url"))
                    .value(json!(config.arrival_video_url)),
            )
            .into(),
        Field::new()
            .name(json!("global_note"))
            .label(json!("i18n:host.note.label"))
            .child(
                TextArea::new()
                    .name(json!("global_note"))
                    .value(json!(config.global_note)),
            )
            .into(),
    ];

    for index in 0..STEP_SLOTS {
        push_step_slot(&mut form_children, index, steps.get(index));
    }

    form_children.push(
        Text::new()
            .text(json!("i18n:host.main.help"))
            .variant(json!("caption"))
            .into(),
    );
    form_children.push(
        Button::new()
            .label(json!("i18n:host.save"))
            .action(save_action)
            .into(),
    );

    Surface::new(
        Page::new()
            .title(json!("i18n:surface.host.main.title"))
            .child(
                Text::new()
                    .text(json!("i18n:surface.host.main.subtitle"))
                    .variant(json!("body")),
            )
            .child(Form::new().children(form_children)),
    )
    .with_id("main")
}

fn steps_to_submit(steps: &[AccessStep]) -> Vec<serde_json::Value> {
    steps
        .iter()
        .map(|s| {
            json!({
                "kind": s.kind.clone().unwrap_or_default(),
                "title_fr": s.title.fr,
                "title_en": s.title.en,
                "detail_fr": s.detail.as_ref().map(|d| d.fr.clone()).unwrap_or_default(),
                "detail_en": s.detail.as_ref().map(|d| d.en.clone()).unwrap_or_default(),
            })
        })
        .collect()
}

fn push_step_slot(children: &mut Vec<Component>, index: usize, step: Option<&AccessStep>) {
    let slot = index + 1;
    let kind = step.and_then(|s| s.kind.as_deref()).unwrap_or("");
    let title_fr = step.map(|s| s.title.fr.as_str()).unwrap_or("");
    let title_en = step.map(|s| s.title.en.as_str()).unwrap_or("");
    let detail_fr = step
        .and_then(|s| s.detail.as_ref())
        .map(|d| d.fr.as_str())
        .unwrap_or("");
    let detail_en = step
        .and_then(|s| s.detail.as_ref())
        .map(|d| d.en.as_str())
        .unwrap_or("");

    children.push(
        Text::new()
            .text(json!(format!("i18n:host.step.slot{slot}")))
            .variant(json!("caption"))
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("steps.{index}.kind")))
            .label(json!("i18n:host.step.kind"))
            .child(
                Select::new()
                    .name(json!(format!("steps.{index}.kind")))
                    .options(json!([
                        {"value": "", "label": "i18n:host.step.kind.other"},
                        {"value": "parking", "label": "i18n:host.step.kind.parking"},
                        {"value": "door", "label": "i18n:host.step.kind.door"},
                        {"value": "elevator", "label": "i18n:host.step.kind.elevator"}
                    ]))
                    .value(json!(kind)),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("steps.{index}.title_fr")))
            .label(json!("i18n:host.step.titleFr"))
            .child(
                TextInput::new()
                    .name(json!(format!("steps.{index}.title_fr")))
                    .value(json!(title_fr)),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("steps.{index}.title_en")))
            .label(json!("i18n:host.step.titleEn"))
            .child(
                TextInput::new()
                    .name(json!(format!("steps.{index}.title_en")))
                    .value(json!(title_en)),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("steps.{index}.detail_fr")))
            .label(json!("i18n:host.step.detailFr"))
            .child(
                TextInput::new()
                    .name(json!(format!("steps.{index}.detail_fr")))
                    .value(json!(detail_fr)),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("steps.{index}.detail_en")))
            .label(json!("i18n:host.step.detailEn"))
            .child(
                TextInput::new()
                    .name(json!(format!("steps.{index}.detail_en")))
                    .value(json!(detail_en)),
            )
            .into(),
    );
}
