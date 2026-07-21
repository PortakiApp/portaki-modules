//! Host dashboard surfaces.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{Button, Field, Form, Page, Select, Text, TextArea, TextInput};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use crate::config::{color_hex_to_name, load_config, BinRow, Localized};

const BIN_SLOTS: usize = 6;

/// Host configuration page — structured bin slots + collection schedule.
#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let lang = Localized::lang_code(&ctx.locale);
    let config = load_config().unwrap_or_default();
    let bins = config.parse_bins();
    let collection_schedule = config.collection_schedule.get(&lang).to_string();

    let submit_args = json!({
        "bins": bins_to_submit(&bins, &lang),
        "collection_schedule": collection_schedule,
    });
    let save_action = serde_json::to_value(Action::command(
        "waste-recycling",
        "updateConfig",
        submit_args,
    ))
    .unwrap_or(json!({}));

    let mut form_children: Vec<Component> = Vec::new();
    for index in 0..BIN_SLOTS {
        push_bin_slot(&mut form_children, index, bins.get(index), &lang);
    }
    form_children.push(
        Field::new()
            .name(json!("collection_schedule"))
            .label(json!("i18n:host.schedule.label"))
            .child(
                TextArea::new()
                    .name(json!("collection_schedule"))
                    .value(json!(collection_schedule))
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

fn bins_to_submit(bins: &[BinRow], lang: &str) -> Vec<serde_json::Value> {
    bins.iter()
        .map(|bin| {
            let items = bin
                .items
                .first()
                .map(|item| item.get(lang))
                .unwrap_or("")
                .to_string();
            json!({
                "title": bin.title.get(lang),
                "items": items,
                "color": color_hex_to_name(bin.color.as_deref()),
            })
        })
        .collect()
}

fn push_bin_slot(children: &mut Vec<Component>, index: usize, bin: Option<&BinRow>, lang: &str) {
    let slot = index + 1;
    let title = bin.map(|b| b.title.get(lang)).unwrap_or("");
    let items = bin
        .and_then(|b| b.items.first())
        .map(|item| item.get(lang))
        .unwrap_or("");
    let color = color_hex_to_name(bin.and_then(|b| b.color.as_deref()));

    children.push(
        Text::new()
            .text(json!(format!("i18n:host.bin.slot{slot}")))
            .variant(json!("caption"))
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("bins.{index}.title")))
            .label(json!("i18n:host.bin.title"))
            .child(
                TextInput::new()
                    .name(json!(format!("bins.{index}.title")))
                    .value(json!(title)),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("bins.{index}.items")))
            .label(json!("i18n:host.bin.items"))
            .child(
                TextInput::new()
                    .name(json!(format!("bins.{index}.items")))
                    .value(json!(items))
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
