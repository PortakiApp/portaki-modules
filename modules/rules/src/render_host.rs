//! Host dashboard surfaces.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{Button, Field, Form, Page, Select, Text, TextInput};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use crate::content::{RuleItem, RulesPayload};
use crate::store;

const ITEM_SLOTS: usize = 6;

/// Host editor — structured rule slots (workspace tab `rules`).
#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let _ = ctx;
    let row = store::load_content().ok().flatten();
    let fr = row
        .as_ref()
        .map(|r| RulesPayload::parse(&r.content_fr))
        .unwrap_or_else(default_fr);
    let en = row
        .as_ref()
        .map(|r| RulesPayload::parse(&r.content_en))
        .unwrap_or_else(default_en);

    let submit_args = json!({
        "items": items_to_submit(&fr, &en),
    });
    let save_action = serde_json::to_value(Action::command("rules", "saveContent", submit_args))
        .unwrap_or(json!({}));

    let mut form_children: Vec<Component> = Vec::new();
    for index in 0..ITEM_SLOTS {
        push_rule_slot(
            &mut form_children,
            index,
            fr.items.get(index),
            en.items.get(index),
        );
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

fn default_fr() -> RulesPayload {
    RulesPayload {
        items: vec![
            RuleItem {
                icon: "clock-circle".into(),
                title: "Calme après 22 h".into(),
                subtitle: "Merci pour le voisinage".into(),
            },
            RuleItem {
                icon: "x".into(),
                title: "Logement non-fumeur".into(),
                subtitle: "Terrasse autorisée".into(),
            },
        ],
    }
}

fn default_en() -> RulesPayload {
    RulesPayload {
        items: vec![
            RuleItem {
                icon: "clock-circle".into(),
                title: "Quiet after 10 pm".into(),
                subtitle: "Please respect neighbours".into(),
            },
            RuleItem {
                icon: "x".into(),
                title: "Non-smoking property".into(),
                subtitle: "Terrace allowed".into(),
            },
        ],
    }
}

fn items_to_submit(fr: &RulesPayload, en: &RulesPayload) -> Vec<serde_json::Value> {
    let len = fr.items.len().max(en.items.len());
    (0..len)
        .map(|i| {
            let fr_item = fr.items.get(i);
            let en_item = en.items.get(i);
            json!({
                "icon": fr_item
                    .map(|r| r.icon.as_str())
                    .filter(|s| !s.is_empty())
                    .or_else(|| en_item.map(|r| r.icon.as_str()).filter(|s| !s.is_empty()))
                    .unwrap_or(""),
                "title_fr": fr_item.map(|r| r.title.as_str()).unwrap_or(""),
                "subtitle_fr": fr_item.map(|r| r.subtitle.as_str()).unwrap_or(""),
                "title_en": en_item.map(|r| r.title.as_str()).unwrap_or(""),
                "subtitle_en": en_item.map(|r| r.subtitle.as_str()).unwrap_or(""),
            })
        })
        .collect()
}

fn push_rule_slot(
    children: &mut Vec<Component>,
    index: usize,
    fr: Option<&RuleItem>,
    en: Option<&RuleItem>,
) {
    let slot = index + 1;
    let icon = fr
        .map(|r| r.icon.as_str())
        .filter(|s| !s.is_empty())
        .or_else(|| en.map(|r| r.icon.as_str()).filter(|s| !s.is_empty()))
        .unwrap_or("check-circle");

    children.push(
        Text::new()
            .text(json!(format!("i18n:host.rule.slot{slot}")))
            .variant(json!("caption"))
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("items.{index}.icon")))
            .label(json!("i18n:host.rule.icon"))
            .child(
                Select::new()
                    .name(json!(format!("items.{index}.icon")))
                    .options(json!([
                        {"value": "clock-circle", "label": "i18n:host.rule.icon.quiet"},
                        {"value": "x", "label": "i18n:host.rule.icon.no"},
                        {"value": "users", "label": "i18n:host.rule.icon.guests"},
                        {"value": "check-circle", "label": "i18n:host.rule.icon.ok"},
                        {"value": "paw-print", "label": "i18n:host.rule.icon.pets"},
                        {"value": "volume-x", "label": "i18n:host.rule.icon.noise"}
                    ]))
                    .value(json!(icon)),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("items.{index}.title_fr")))
            .label(json!("i18n:host.rule.titleFr"))
            .child(
                TextInput::new()
                    .name(json!(format!("items.{index}.title_fr")))
                    .value(json!(fr.map(|r| r.title.as_str()).unwrap_or(""))),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("items.{index}.subtitle_fr")))
            .label(json!("i18n:host.rule.subtitleFr"))
            .child(
                TextInput::new()
                    .name(json!(format!("items.{index}.subtitle_fr")))
                    .value(json!(fr.map(|r| r.subtitle.as_str()).unwrap_or(""))),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("items.{index}.title_en")))
            .label(json!("i18n:host.rule.titleEn"))
            .child(
                TextInput::new()
                    .name(json!(format!("items.{index}.title_en")))
                    .value(json!(en.map(|r| r.title.as_str()).unwrap_or(""))),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("items.{index}.subtitle_en")))
            .label(json!("i18n:host.rule.subtitleEn"))
            .child(
                TextInput::new()
                    .name(json!(format!("items.{index}.subtitle_en")))
                    .value(json!(en.map(|r| r.subtitle.as_str()).unwrap_or(""))),
            )
            .into(),
    );
}
