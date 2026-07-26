//! Host dashboard surface — design `rules-editor-v1` (Wasm SDUI).
//!
//! Dashboard: section « Règles du logement » + dynamic rule rows (StepList).
//! Save chrome is owned by the workspace tab (`updateConfig`).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{
    Card, Field, Form, Page, Select, Stack, StepList, TextInput,
};
use portaki_sdk::sdui::surface::Surface;
use serde::Serialize;

use crate::content::{RuleItem, RulesBundle, RulesPayload};
use crate::store;

/// Design / mobile upper bound — « ajoutez-en autant que nécessaire », capped.
const ITEM_SLOTS: usize = 12;

/// Host editor — dynamic bilingual rule rows for the active `ctx.locale`.
///
/// No in-form Save — workspace header owns Enregistrer → `updateConfig`.
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

    let items_count = draft_items_count(&ctx, &payload);
    let mut rows: Vec<Component> = Vec::new();
    for index in 0..items_count {
        rows.push(rule_row(index, payload.items.get(index)));
    }

    Surface::new(
        Page::new().child(Form::new().child(
            Card::new()
                .title("i18n:host.section.title")
                .subtitle("i18n:host.section.subtitle")
                .icon("scale")
                .child(
                    StepList::new()
                        .addLabel("i18n:host.rules.add")
                        .removeLabel("i18n:host.rules.remove")
                        .emptyTitle("i18n:host.rules.emptyTitle")
                        .emptyDescription("i18n:host.rules.emptyDescription")
                        .itemKeyPrefix("items")
                        .addAction(emit_input(ItemsCountInput {
                            items_count: (items_count + 1).min(ITEM_SLOTS),
                        }))
                        .children(rows),
                ),
        )),
    )
    .with_id(crate::ids::HOST_MAIN)
}

fn draft_items_count(ctx: &HostContext, payload: &RulesPayload) -> usize {
    if let Some(n) = ctx.input_u64("items_count") {
        return (n as usize).clamp(1, ITEM_SLOTS);
    }
    let existing = payload
        .items
        .iter()
        .filter(|item| !item.title.trim().is_empty())
        .count();
    if existing == 0 {
        // Empty store → seed the four design defaults in the form.
        default_for_lang("fr").items.len().min(ITEM_SLOTS)
    } else {
        existing.min(ITEM_SLOTS)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct ItemsCountInput {
    items_count: usize,
}

fn emit_input(payload: impl Serialize) -> Action {
    Action::emit(contracts::shell::SURFACE_INPUT, Some(json_value(payload)))
}

/// Design defaults — Portaki Dashboard `editorRules` / Guest `rules` block.
pub(crate) fn default_for_lang(lang: &str) -> RulesPayload {
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
                RuleItem {
                    icon: "gift".into(),
                    title: "Pets on request".into(),
                    subtitle: "Let us know before arrival".into(),
                },
                RuleItem {
                    icon: "users".into(),
                    title: "No parties".into(),
                    subtitle: "Respect the guest count".into(),
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
                RuleItem {
                    icon: "gift".into(),
                    title: "Animaux sur demande".into(),
                    subtitle: "Prévenez-nous avant l'arrivée".into(),
                },
                RuleItem {
                    icon: "users".into(),
                    title: "Pas de fête".into(),
                    subtitle: "Respect du nombre de voyageurs".into(),
                },
            ],
        }
    }
}

fn rule_row(index: usize, item: Option<&RuleItem>) -> Component {
    let icon = item
        .map(|r| r.icon.as_str())
        .filter(|s| !s.is_empty())
        .map(normalize_icon)
        .unwrap_or("check-circle");

    Stack::new()
        .id(format!("rule-{index}"))
        .gap(10.0)
        .children(vec![
            Field::new()
                .name(format!("items.{index}.icon"))
                .label("i18n:host.rule.icon")
                .child(
                    Select::new()
                        .name(format!("items.{index}.icon"))
                        .options(rule_icon_options())
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

/// Design `ruleIcons()` — clock-circle, x, users, check-circle, gift, minus.
fn rule_icon_options() -> Vec<ChoiceOption> {
    vec![
        ChoiceOption::new("clock-circle", "i18n:host.rule.icon.quiet"),
        ChoiceOption::new("x", "i18n:host.rule.icon.no"),
        ChoiceOption::new("users", "i18n:host.rule.icon.guests"),
        ChoiceOption::new("check-circle", "i18n:host.rule.icon.ok"),
        ChoiceOption::new("gift", "i18n:host.rule.icon.pets"),
        ChoiceOption::new("minus", "i18n:host.rule.icon.noise"),
    ]
}

/// Map legacy stored icons onto the design set.
fn normalize_icon(icon: &str) -> &str {
    match icon {
        "paw-print" | "pets" => "gift",
        "volume-x" | "volume-2" | "noise" => "minus",
        other => other,
    }
}
