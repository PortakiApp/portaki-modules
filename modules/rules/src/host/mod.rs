//! Host dashboard surface — design `rules-editor-v1` (Wasm SDUI).
//!
//! Dashboard: section « Règles du logement » + dynamic rule rows (StepList).
//! Save chrome is owned by the workspace tab (`updateConfig`).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{
    Card, Field, FieldHint, Form, NumberInput, Page, Select, Stack, StepList, TextInput,
};
use portaki_sdk::sdui::surface::Surface;
use serde::Serialize;

use crate::content::{
    RuleItem, RuleStatus, RulesBundle, RulesPayload, MAX_CARD_LIMIT, MAX_RULES, MIN_CARD_LIMIT,
    THEMES,
};
use crate::store;

/// Design / mobile upper bound — « ajoutez-en autant que nécessaire », capped.
const ITEM_SLOTS: usize = MAX_RULES;

/// Host editor — dynamic bilingual rule rows for the active `ctx.locale`.
///
/// No in-form Save — workspace header owns Enregistrer → `updateConfig`.
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::RulesEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Scale
)]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let lang = RulesBundle::lang_code(&ctx.locale);
    let row = store::load_content().ok().flatten();
    let bundle = row
        .as_ref()
        .map(|r| RulesBundle::from_row(&r.content_fr, &r.content_en))
        .unwrap_or_default();
    let payload = {
        let current = bundle.get(&lang);
        if current.items.is_empty() {
            default_for_lang(&lang)
        } else {
            current
        }
    };

    let items_count = draft_items_count(&ctx, &payload);
    let mut rows: Vec<Component> = Vec::new();
    for index in 0..items_count {
        rows.push(rule_row(index, payload.items.get(index), &ctx));
    }

    Surface::new(
        Page::new().child(
            Form::new().child(
                Stack::new()
                    .gap(16.0)
                    .child(display_card(&bundle, &ctx))
                    .child(
                        Card::new()
                            .title("i18n:host.section.title")
                            .subtitle("i18n:host.section.subtitle")
                            .icon(IconName::Scale)
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
                    ),
            ),
        ),
    )
    .with_id(MAIN)
}

/// §2.1 Affichage : combien de règles sur la carte, et où se règle la signature.
fn display_card(bundle: &RulesBundle, ctx: &HostContext) -> Component {
    let stored = bundle.card_limit;
    let mut field = Field::new()
        .name("card_limit")
        .label("i18n:host.cardLimit.label")
        .child(
            NumberInput::new()
                .name("card_limit")
                .min(f64::from(MIN_CARD_LIMIT))
                .max(f64::from(MAX_CARD_LIMIT))
                .value(stored.map_or(bundle.card_limit() as f64, f64::from)),
        );
    // La valeur saisie, même hors bornes : c'est elle que l'erreur désigne.
    if let Some(error) = stored.and_then(|n| {
        portaki_sdk::config::check::between(
            f64::from(n),
            f64::from(MIN_CARD_LIMIT),
            f64::from(MAX_CARD_LIMIT),
        )
    }) {
        field = field.error(error.get(&ctx.locale).to_string());
    }
    Card::new()
        .title("i18n:host.display.title")
        .icon(IconName::Home)
        .child(field)
        .child(FieldHint::new().text("i18n:host.cardLimit.hint"))
        // Faire accepter le règlement se règle dans Formalités : on le dit ici, sans le dupliquer.
        .child(FieldHint::new().text("i18n:host.acceptance.hint"))
        .into()
}

fn draft_items_count(ctx: &HostContext, payload: &RulesPayload) -> usize {
    if let Some(n) = ctx.input_u64("items_count") {
        return (n as usize).clamp(1, ITEM_SLOTS);
    }
    let existing = payload.items.iter().filter(|item| !item.is_blank()).count();
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
    // Le modèle de la spec (§4) : silence, tabac, animaux, fêtes.
    let rule =
        |icon: &str, title: &str, subtitle: &str, status, theme: &str, hours: &str| RuleItem {
            icon: icon.into(),
            title: title.into(),
            subtitle: subtitle.into(),
            status,
            theme: theme.into(),
            hours: hours.into(),
        };
    let items = if lang == "en" {
        vec![
            rule(
                "clock-circle",
                "Quiet hours",
                "Please respect neighbours",
                RuleStatus::Important,
                "noise",
                "22:00 – 08:00",
            ),
            rule(
                "x",
                "Non-smoking property",
                "Terrace allowed",
                RuleStatus::Forbidden,
                "smoking",
                "",
            ),
            rule(
                "gift",
                "Pets on request",
                "Let us know before arrival",
                RuleStatus::Allowed,
                "pets",
                "",
            ),
            rule(
                "users",
                "No parties",
                "Respect the guest count",
                RuleStatus::Forbidden,
                "parties",
                "",
            ),
        ]
    } else {
        vec![
            rule(
                "clock-circle",
                "Silence",
                "Merci pour le voisinage",
                RuleStatus::Important,
                "noise",
                "22:00 – 08:00",
            ),
            rule(
                "x",
                "Logement non-fumeur",
                "Terrasse autorisée",
                RuleStatus::Forbidden,
                "smoking",
                "",
            ),
            rule(
                "gift",
                "Animaux sur demande",
                "Prévenez-nous avant l'arrivée",
                RuleStatus::Allowed,
                "pets",
                "",
            ),
            rule(
                "users",
                "Pas de fête",
                "Respect du nombre de voyageurs",
                RuleStatus::Forbidden,
                "parties",
                "",
            ),
        ]
    };
    RulesPayload {
        items,
        ..RulesPayload::default()
    }
}

fn rule_row(index: usize, item: Option<&RuleItem>, ctx: &HostContext) -> Component {
    let problems = item.map(RuleItem::problems).unwrap_or_default();
    let named = |key: &str| {
        let field = Field::new().name(format!("items.{index}.{key}"));
        match problems.iter().find(|(name, _)| *name == key) {
            Some((_, error)) => field.error(error.get(&ctx.locale).to_string()),
            None => field,
        }
    };
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
            named("title")
                .label("i18n:host.rule.title")
                .child(
                    TextInput::new()
                        .name(format!("items.{index}.title"))
                        .value(item.map(|r| r.title.as_str()).unwrap_or("")),
                )
                .into(),
            named("subtitle")
                .label("i18n:host.rule.subtitle")
                .child(
                    TextInput::new()
                        .name(format!("items.{index}.subtitle"))
                        .value(item.map(|r| r.subtitle.as_str()).unwrap_or("")),
                )
                .into(),
            Field::new()
                .name(format!("items.{index}.status"))
                .label("i18n:host.rule.status")
                .child(
                    Select::new()
                        .name(format!("items.{index}.status"))
                        .options(rule_status_options())
                        .value(item.map(|r| r.status).unwrap_or_default().as_wire()),
                )
                .into(),
            // Thème : texte libre, pas une liste fermée. Le mockup groupe « Voisinage »,
            // « Piscine », « Extérieur » — un hôte sait nommer ses propres rubriques mieux
            // qu'une énumération écrite ici, et le détail se groupe sur ce qu'il a écrit.
            Field::new()
                .name(format!("items.{index}.theme"))
                .label("i18n:host.rule.theme")
                .child(
                    Select::new()
                        .name(format!("items.{index}.theme"))
                        .options(
                            THEMES
                                .iter()
                                .map(|key| {
                                    ChoiceOption::new(*key, format!("i18n:rule.theme.{key}"))
                                })
                                .collect(),
                        )
                        .value(item.and_then(RuleItem::theme_key).unwrap_or("other")),
                )
                .into(),
            Field::new()
                .name(format!("items.{index}.hours"))
                .label("i18n:host.rule.hours")
                .child(
                    TextInput::new()
                        .name(format!("items.{index}.hours"))
                        .placeholder("22:00 – 08:00")
                        .value(item.map(|r| r.hours.as_str()).unwrap_or("")),
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

/// Type d'une règle (spec Règlement §2.2) : Interdit · Important · Autorisé · Information.
fn rule_status_options() -> Vec<ChoiceOption> {
    vec![
        ChoiceOption::new("forbidden", "i18n:rule.status.forbidden"),
        ChoiceOption::new("important", "i18n:rule.status.important"),
        ChoiceOption::new("allowed", "i18n:rule.status.allowed"),
        ChoiceOption::new("neutral", "i18n:rule.status.neutral"),
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
