//! Host dashboard surface — the settings page (spec Règlement §2), mode « page ».
//!
//! Two `Section`s: « Affichage » (rules on the home card, where signing is set), then « Règles »,
//! an ordered table (drag = booklet order) with a rule « Sur la carte d'accueil » after the first
//! `card_limit` rows. A row opens in a centered dialog (`panel: "dialog"`), its fields named
//! `rules.<i>.<key>`. Every edit autosaves through `updateConfig`; a broken rule shows under its
//! field and blocks « Publier » (`publishReadiness`), never the save.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::{ChoiceListLayout, Tone};
use portaki_sdk::sdui::primitives::{
    Button, ChoiceList, DataTable, Field, FieldHint, Form, InlineNotice, NumberInput, Page,
    Section, Select, Stack, Text, TextArea, TextInput, TimeRange,
};
use portaki_sdk::sdui::surface::Surface;
use serde_json::{json, Map, Value};

use crate::content::{
    range_wire, RuleItem, RuleStatus, RulesBundle, RulesPayload, MAX_CARD_LIMIT, MAX_RULES,
    MIN_CARD_LIMIT, THEMES,
};
use crate::store;

/// Les règles ajoutées pas encore enregistrées : leurs id, séparés par des virgules.
const NEW_RULES: &str = "newRules";

/// Réglages du module (page) : affichage, puis règles.
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::RulesEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Scale
)]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let bundle = store::load_content()
        .ok()
        .flatten()
        .map(|r| RulesBundle::from_row(&r.content_fr, &r.content_en))
        .unwrap_or_default();
    let payload = bundle.get(&RulesBundle::lang_code(&ctx.locale));
    let rows = rows(&payload, &ctx);
    Surface::new(
        Page::new().child(
            Form::new()
                .child(display_section(&bundle, &ctx))
                .child(rules_section(&bundle, &rows, &ctx)),
        ),
    )
    .with_id(MAIN)
}

/// Les lignes du tableau : les règles enregistrées, puis celles qu'« Ajouter » a ouvertes.
fn rows(payload: &RulesPayload, ctx: &HostContext) -> Vec<RuleItem> {
    let mut rows = payload.items.clone();
    for id in ctx.input_str(NEW_RULES).unwrap_or_default().split(',') {
        let id = id.trim();
        if !id.is_empty() && payload.find(id).is_none() {
            rows.push(RuleItem {
                id: id.to_string(),
                ..RuleItem::default()
            });
        }
    }
    rows
}

/// §2.1 Affichage : combien de règles sur la carte, et où se règle la signature.
fn display_section(bundle: &RulesBundle, ctx: &HostContext) -> Component {
    let stored = bundle.card_limit;
    let mut field = Field::new()
        .name("card_limit")
        .label("i18n:host.cardLimit.label")
        .required(true);
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
    let card_limit = field
        .child(
            NumberInput::new()
                .name("card_limit")
                .min(f64::from(MIN_CARD_LIMIT))
                .max(f64::from(MAX_CARD_LIMIT))
                .value(stored.map_or(bundle.card_limit() as f64, f64::from)),
        )
        .child(FieldHint::new().text("i18n:host.cardLimit.hint"));
    // Faire accepter le règlement ne se règle pas ici : on le dit, en lecture seule.
    // ponytail: pas de lien — aucune page « Formalités » du logement n'existe encore côté tableau
    // de bord ; ajouter un `Link` vers sa route quand elle existera.
    let acceptance = Field::new()
        .name("acceptance")
        .label("i18n:host.acceptance.label")
        .child(
            Text::new()
                .text("i18n:host.acceptance.hint")
                .variant(TextVariant::Body),
        );
    Section::new()
        .title("i18n:host.display.title")
        .child(card_limit)
        .child(acceptance)
        .into()
}

/// §2.2 Règles : le tableau ordonnable, le dialogue de chaque ligne, « Partir d'un modèle ».
fn rules_section(bundle: &RulesBundle, rows: &[RuleItem], ctx: &HostContext) -> Component {
    let mut children: Vec<Component> = vec![FieldHint::new().text("i18n:host.rules.hint").into()];
    if rows.len() > MAX_RULES {
        children.push(
            InlineNotice::new()
                .tone(Tone::Danger)
                .message("i18n:host.rules.tooMany")
                .into(),
        );
    }

    let mut table = DataTable::new()
        .rowName("rules")
        .sortable(true)
        .dividerAfter(bundle.card_limit() as u32)
        .dividerLabel("i18n:host.rules.divider")
        .panel("dialog")
        .columns(json!([
            { "key": "glyph", "label": "i18n:host.column.status", "kind": "pill" },
            { "key": "title", "label": "i18n:host.rule.title", "sub": "detail" },
            { "key": "theme", "label": "i18n:host.rule.theme" },
            { "key": "hours", "label": "i18n:host.rule.hours", "mono": true },
        ]))
        .rows(Value::Array(rows.iter().map(row).collect()))
        .children(
            rows.iter()
                .enumerate()
                .map(|(index, rule)| rule_panel(index, rule, ctx))
                .collect(),
        );
    if rows.len() < MAX_RULES {
        let mut added: Vec<String> = rows
            .iter()
            .filter(|r| r.title.is_empty() && r.subtitle.is_empty())
            .map(|r| r.id.clone())
            .collect();
        added.push(uuid::Uuid::new_v4().to_string());
        let mut input = Map::new();
        input.insert(NEW_RULES.into(), Value::String(added.join(",")));
        table = table
            .addLabel("i18n:host.rules.add")
            .addAction(Action::emit(
                contracts::shell::SURFACE_INPUT,
                Some(Value::Object(input)),
            ));
    }
    children.push(table.into());

    // §4 : un règlement vide part d'un modèle (silence, animaux, tabac, fêtes, visiteurs).
    if rows.is_empty() {
        children.push(
            Button::new()
                .label("i18n:host.rules.template")
                .variant(ButtonVariant::Outline)
                .action(crate::ids::module_id().command(
                    crate::commands::APPLY_TEMPLATE,
                    crate::commands::ApplyTemplateArgs {},
                ))
                .into(),
        );
    }

    Section::new()
        .title("i18n:host.rules.title")
        .subtitle(format!("{} / {MAX_RULES}", rows.len()))
        .children(children)
        .into()
}

/// Une ligne du tableau : glyphe, règle (précision dessous), thème, horaires.
fn row(rule: &RuleItem) -> Value {
    let theme = match rule.theme_key() {
        Some(key) => format!("i18n:rule.theme.{key}"),
        None => rule.theme.trim().to_string(),
    };
    let hours = rule.hours_label();
    json!({
        "id": rule.id,
        "glyph": if rule.title.is_empty() { "" } else { rule.status.glyph() },
        "title": rule.title,
        "detail": rule.subtitle,
        "theme": theme,
        "hours": if hours.is_empty() { "—".to_string() } else { hours },
    })
}

/// Le dialogue d'une ligne : Règle, Précision, Type, Thème, Horaires (§2.2).
fn rule_panel(index: usize, rule: &RuleItem, ctx: &HostContext) -> Component {
    let name = |key: &str| format!("rules.{index}.{key}");
    let problems = rule.problems();
    let field = |key: &str, label: &str| {
        let field = Field::new().name(name(key)).label(label);
        match problems.iter().find(|(name, _)| *name == key) {
            Some((_, error)) => field.error(error.get(&ctx.locale).to_string()),
            None => field,
        }
    };

    // Le thème : celui de la liste ; un ancien texte libre, ou rien, prend celui que le texte
    // suggère (§4). Une suggestion qui diffère du thème choisi s'affiche dessous, sans rien forcer.
    let theme = rule
        .theme_key()
        .or_else(|| rule.suggested_theme())
        .unwrap_or("other");
    let mut theme_field = field("theme", "i18n:host.rule.theme").child(
        Select::new()
            .name(name("theme"))
            .options(
                THEMES
                    .iter()
                    .map(|key| ChoiceOption::new(*key, format!("i18n:rule.theme.{key}")))
                    .collect(),
            )
            .value(theme),
    );
    if let Some(suggested) = rule.suggested_theme().filter(|s| *s != theme) {
        let label = t!(&format!("rule.theme.{suggested}")).unwrap_or_default();
        if let Ok(hint) = t!("host.rule.theme.suggested", theme = label.as_str()) {
            theme_field = theme_field.child(FieldHint::new().text(hint));
        }
    }

    // Les heures : la plage du `TimeRange`. Un ancien texte illisible n'entre pas dans le champ ; il
    // reste affiché dessous, et s'efface au premier choix d'une plage.
    let wire = range_wire(&rule.hours);
    let readable = portaki_sdk::sdui::hours::parse_range(&wire).is_some();
    let mut hours_field = field("hours", "i18n:host.rule.hours").child(
        TimeRange::new()
            .name(name("hours"))
            .value(if readable { wire.as_str() } else { "" }),
    );
    if !readable && !rule.hours.trim().is_empty() {
        hours_field = hours_field.child(FieldHint::new().text(rule.hours.trim()));
    }

    let statuses = [
        (RuleStatus::Forbidden, IconName::X),
        (RuleStatus::Important, IconName::TriangleAlert),
        (RuleStatus::Allowed, IconName::Check),
        (RuleStatus::Neutral, IconName::Info),
    ];

    Stack::new()
        .gap(14.0)
        .children(vec![
            field("title", "i18n:host.rule.title")
                .required(true)
                .child(
                    TextInput::new()
                        .name(name("title"))
                        .value(rule.title.as_str())
                        .placeholder("i18n:host.rule.title.placeholder"),
                )
                .into(),
            field("detail", "i18n:host.rule.detail")
                .child(
                    TextArea::new()
                        .name(name("detail"))
                        .value(rule.subtitle.as_str()),
                )
                .into(),
            field("status", "i18n:host.rule.status")
                .child(
                    ChoiceList::new()
                        .name(name("status"))
                        .layout(ChoiceListLayout::Cards)
                        .value(rule.status.as_wire())
                        .choices(
                            statuses
                                .iter()
                                .map(|(status, icon)| {
                                    ChoiceOption::new(
                                        status.as_wire(),
                                        format!("i18n:rule.status.{}", status.as_wire()),
                                    )
                                    .icon(*icon)
                                })
                                .collect(),
                        ),
                )
                .into(),
            theme_field.into(),
            hours_field.into(),
        ])
        .into()
}
