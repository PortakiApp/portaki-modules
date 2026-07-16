//! Host dashboard surfaces.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{Button, Field, Form, Page, Text, TextArea, TextInput};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use crate::config::{load_config, SpotRow};

const SPOT_SLOTS: usize = 6;

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let _ = ctx;
    let config = load_config().unwrap_or_default();
    let spots = config.parse_spots();

    let submit_args = json!({
        "spots": spots_to_submit(&spots),
        "disclaimer": config.disclaimer,
    });
    let save_action =
        serde_json::to_value(Action::command("local-guide", "updateConfig", submit_args))
            .unwrap_or(json!({}));

    let mut form_children: Vec<Component> = Vec::new();
    for index in 0..SPOT_SLOTS {
        push_spot_slot(&mut form_children, index, spots.get(index));
    }
    form_children.push(
        Field::new()
            .name(json!("disclaimer"))
            .label(json!("i18n:host.disclaimer.label"))
            .child(
                TextArea::new()
                    .name(json!("disclaimer"))
                    .value(json!(config.disclaimer))
                    .placeholder(json!("i18n:host.disclaimer.placeholder")),
            )
            .into(),
    );
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

fn spots_to_submit(spots: &[SpotRow]) -> Vec<serde_json::Value> {
    spots
        .iter()
        .map(|s| {
            let name = if !s.title.fr.trim().is_empty() {
                s.title.fr.clone()
            } else {
                s.title.en.clone()
            };
            let description = s
                .detail
                .as_ref()
                .map(|d| {
                    if !d.fr.trim().is_empty() {
                        d.fr.clone()
                    } else {
                        d.en.clone()
                    }
                })
                .unwrap_or_default();
            json!({
                "name": name,
                "category": s.category.clone().unwrap_or_default(),
                "distance": s.distance.clone().unwrap_or_default(),
                "tag": s.tag.clone().unwrap_or_default(),
                "description": description,
            })
        })
        .collect()
}

fn push_spot_slot(children: &mut Vec<Component>, index: usize, spot: Option<&SpotRow>) {
    let slot = index + 1;
    let name = spot
        .map(|s| {
            if !s.title.fr.trim().is_empty() {
                s.title.fr.as_str()
            } else {
                s.title.en.as_str()
            }
        })
        .unwrap_or("");
    let category = spot.and_then(|s| s.category.as_deref()).unwrap_or("");
    let distance = spot.and_then(|s| s.distance.as_deref()).unwrap_or("");
    let tag = spot.and_then(|s| s.tag.as_deref()).unwrap_or("");
    let description = spot
        .and_then(|s| s.detail.as_ref())
        .map(|d| {
            if !d.fr.trim().is_empty() {
                d.fr.as_str()
            } else {
                d.en.as_str()
            }
        })
        .unwrap_or("");

    children.push(
        Text::new()
            .text(json!(format!("i18n:host.spot.slot{slot}")))
            .variant(json!("caption"))
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("spots.{index}.name")))
            .label(json!("i18n:host.spot.name"))
            .child(
                TextInput::new()
                    .name(json!(format!("spots.{index}.name")))
                    .value(json!(name)),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("spots.{index}.category")))
            .label(json!("i18n:host.spot.category"))
            .child(
                TextInput::new()
                    .name(json!(format!("spots.{index}.category")))
                    .value(json!(category)),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("spots.{index}.distance")))
            .label(json!("i18n:host.spot.distance"))
            .child(
                TextInput::new()
                    .name(json!(format!("spots.{index}.distance")))
                    .value(json!(distance)),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("spots.{index}.tag")))
            .label(json!("i18n:host.spot.tag"))
            .child(
                TextInput::new()
                    .name(json!(format!("spots.{index}.tag")))
                    .value(json!(tag)),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("spots.{index}.description")))
            .label(json!("i18n:host.spot.description"))
            .child(
                TextArea::new()
                    .name(json!(format!("spots.{index}.description")))
                    .value(json!(description)),
            )
            .into(),
    );
}
