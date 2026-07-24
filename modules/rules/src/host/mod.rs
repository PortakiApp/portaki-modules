//! Host dashboard surface — design `rules-editor-v1` (Wasm SDUI for mobile + fallback).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Button, Card, Field, Form, Page, Select, Stack, Text, TextInput,
};
use portaki_sdk::sdui::surface::Surface;

use crate::content::{RuleItem, RulesBundle, RulesPayload};
use crate::store;

const ITEM_SLOTS: usize = 6;

/// Host editor — rule cards for the active `ctx.locale`.
#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let lang = RulesBundle::lang_code(&ctx.locale);
    let row = store::load_content().ok().flatten();
    let bundle = row
        .as_ref()
        .map(|r| RulesBundle::from_row(&r.content_fr, &r.content_en))
        .unwrap_or_default();
    let payload = {
        let current = bundle.get(&lang);
        if current.is_empty() {
            default_for_lang(&lang)
        } else {
            current
        }
    };

    let submit_args = crate::commands::SaveContentArgs {
        items: items_to_submit(&payload),
        content_fr: String::new(),
        content_en: String::new(),
    };
    let save_action = crate::ids::module_id().command(crate::ids::SAVE_CONTENT, submit_args);

    let mut cards: Vec<Component> = Vec::new();
    for index in 0..ITEM_SLOTS {
        cards.push(rule_card(index, payload.items.get(index)));
    }
    cards.push(
        Button::new()
            .label("i18n:host.save")
            .tone(Tone::Primary)
            .action(save_action)
            .into(),
    );

    Surface::new(
        Page::new().child(
            Form::new().child(
                Stack::new()
                    .gap(16.0)
                    .children(vec![
                        Text::new()
                            .text("i18n:surface.host.main.subtitle")
                            .variant(TextVariant::Body)
                            .into(),
                        Component::Stack(Stack::new().gap(16.0).children(cards)),
                    ]),
            ),
        ),
    )
    .with_id(crate::ids::HOST_MAIN)
}

fn default_for_lang(lang: &str) -> RulesPayload {
    if lang == "en" {
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
    } else {
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
}

fn items_to_submit(payload: &RulesPayload) -> Vec<crate::commands::RuleItemInput> {
    payload
        .items
        .iter()
        .map(|item| crate::commands::RuleItemInput {
            icon: item.icon.clone(),
            title: item.title.clone(),
            subtitle: item.subtitle.clone(),
            ..Default::default()
        })
        .collect()
}

fn rule_card(index: usize, item: Option<&RuleItem>) -> Component {
    let slot = index + 1;
    let icon = item
        .map(|r| r.icon.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("check-circle");

    Card::new()
        .title(format!("i18n:host.rule.slot{slot}"))
        .icon(icon)
        .children(vec![
            Field::new()
                .name(format!("items.{index}.icon"))
                .label("i18n:host.rule.icon")
                .child(
                    Select::new()
                        .name(format!("items.{index}.icon"))
                        .options(vec![
                            ChoiceOption::new("clock-circle", "i18n:host.rule.icon.quiet"),
                            ChoiceOption::new("x", "i18n:host.rule.icon.no"),
                            ChoiceOption::new("users", "i18n:host.rule.icon.guests"),
                            ChoiceOption::new("check-circle", "i18n:host.rule.icon.ok"),
                            ChoiceOption::new("paw-print", "i18n:host.rule.icon.pets"),
                            ChoiceOption::new("volume-x", "i18n:host.rule.icon.noise"),
                        ])
                        .value(icon),
                )
                .into(),
            Field::new()
                .name(format!("items.{index}.title"))
                .label("i18n:host.rule.title")
                .child(
                    TextInput::new()
                        .name(format!("items.{index}.title"))
                        .value(item.map(|r| r.title.as_str()).unwrap_or("")),
                )
                .into(),
            Field::new()
                .name(format!("items.{index}.subtitle"))
                .label("i18n:host.rule.subtitle")
                .child(
                    TextInput::new()
                        .name(format!("items.{index}.subtitle"))
                        .value(item.map(|r| r.subtitle.as_str()).unwrap_or("")),
                )
                .into(),
        ])
        .into()
}
