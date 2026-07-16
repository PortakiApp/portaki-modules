//! Host dashboard surfaces.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{Button, Field, Form, Page, Select, Text, TextArea, TextInput};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use crate::config::{color_hex_to_name, load_config, BinRow};

const BIN_SLOTS: usize = 6;

/// Host configuration page — structured bin slots + collection schedule.
#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let _ = ctx;
    let config = load_config().unwrap_or_default();
    let bins = config.parse_bins();

    let submit_args = json!({
        "bins": bins_to_submit(&bins),
        "collection_schedule": config.collection_schedule,
    });
    let save_action = serde_json::to_value(Action::command(
        "waste-recycling",
        "updateConfig",
        submit_args,
    ))
    .unwrap_or(json!({}));

    let mut form_children: Vec<Component> = Vec::new();
    for index in 0..BIN_SLOTS {
        let bin = bins.get(index);
        push_bin_slot(&mut form_children, index, bin);
    }
    form_children.push(
        Field::new()
            .name(json!("collection_schedule"))
            .label(json!("i18n:host.schedule.label"))
            .child(
                TextArea::new()
                    .name(json!("collection_schedule"))
                    .value(json!(config.collection_schedule))
                    .placeholder(json!("i18n:host.schedule.placeholder")),
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

fn bins_to_submit(bins: &[BinRow]) -> Vec<serde_json::Value> {
    bins.iter()
        .map(|bin| {
            let items_fr = bin
                .items
                .iter()
                .map(|item| item.fr.as_str())
                .filter(|s| !s.trim().is_empty())
                .collect::<Vec<_>>()
                .join(", ");
            json!({
                "title_fr": bin.title.fr,
                "title_en": bin.title.en,
                "items_fr": items_fr,
                "color": color_hex_to_name(bin.color.as_deref()),
            })
        })
        .collect()
}

fn push_bin_slot(children: &mut Vec<Component>, index: usize, bin: Option<&BinRow>) {
    let slot = index + 1;
    let title_fr = bin.map(|b| b.title.fr.as_str()).unwrap_or("");
    let title_en = bin.map(|b| b.title.en.as_str()).unwrap_or("");
    let items_fr = bin
        .map(|b| {
            b.items
                .iter()
                .map(|item| item.fr.as_str())
                .filter(|s| !s.trim().is_empty())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let color = color_hex_to_name(bin.and_then(|b| b.color.as_deref()));

    children.push(
        Text::new()
            .text(json!(format!("i18n:host.bin.slot{slot}")))
            .variant(json!("caption"))
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("bins.{index}.title_fr")))
            .label(json!("i18n:host.bin.titleFr"))
            .child(
                TextInput::new()
                    .name(json!(format!("bins.{index}.title_fr")))
                    .value(json!(title_fr)),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("bins.{index}.title_en")))
            .label(json!("i18n:host.bin.titleEn"))
            .child(
                TextInput::new()
                    .name(json!(format!("bins.{index}.title_en")))
                    .value(json!(title_en)),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("bins.{index}.items_fr")))
            .label(json!("i18n:host.bin.items"))
            .child(
                TextInput::new()
                    .name(json!(format!("bins.{index}.items_fr")))
                    .value(json!(items_fr))
                    .placeholder(json!("i18n:host.bin.items.placeholder")),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("bins.{index}.color")))
            .label(json!("i18n:host.bin.color"))
            .child(
                Select::new()
                    .name(json!(format!("bins.{index}.color")))
                    .options(json!([
                        {"value": "", "label": "i18n:host.bin.color.none"},
                        {"value": "yellow", "label": "i18n:host.bin.color.yellow"},
                        {"value": "green", "label": "i18n:host.bin.color.green"},
                        {"value": "brown", "label": "i18n:host.bin.color.brown"},
                        {"value": "grey", "label": "i18n:host.bin.color.grey"}
                    ]))
                    .value(json!(color)),
            )
            .into(),
    );
}
