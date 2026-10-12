//! Host settings page — spec Pré-arrivée §2, mode « page ».
//!
//! Three `Section`s, the shell's cards, in the spec's order: arrival time, questions, sending and
//! reminders. The host's own questions are a sortable `DataTable` whose row opens in a dialog
//! (`panel: "dialog"`), its fields named `custom_questions.<i>.<key>`; the options show only for
//! « Choix » (`visibleWhen`). A rule broken shows under its field, the same one
//! `publishReadiness` blocks on; no Save button — the shell autosaves through `updateConfig`.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui;
use portaki_sdk::sdui::common::{ChoiceListLayout, Tone};
use portaki_sdk::sdui::primitives::{
    ChoiceList, DataTable, Field, FieldHint, Form, InlineNotice, Page, Section, Stack, TextInput,
    TimePicker, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;
use serde_json::{json, Map, Value};

use crate::config::{
    CustomQuestion, Deadline, ModuleConfig, QuestionKind, ShowWhen, DEFAULT_LATE_FROM,
    DEFAULT_SLOTS_UNTIL, MAX_CUSTOM_QUESTIONS, MAX_OPTIONS,
};

/// Les questions ajoutées pas encore enregistrées : leurs id, séparés par des virgules.
const NEW_QUESTIONS: &str = "newQuestions";

// « Arrivée tardive · Marie · 23:30 » dans À venir : `crate::tasks`.
#[portaki_sdk::nav(
    placement = HostPlacement::WorkspaceTimelineTask,
    path = "tasks",
    label_key = "catalog.host.tasks",
    icon = IconName::ClockCircle
)]
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::PrearrivalEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Clipboard
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    Ok(Surface::new(
        Page::new().child(
            Form::new()
                .child(arrival_section(&config, &ctx))
                .child(questions_section(&config, &ctx))
                .child(sending_section(&config)),
        ),
    )
    .with_id(MAIN))
}

// ── 2.1 Heure d'arrivée ──────────────────────────────────────────────────────

/// Les créneaux partent de l'heure d'arrivée du logement : une surface hôte ne la reçoit pas
/// (`PropertyContext` n'a pas d'heure d'entrée), donc seule la fin et le pas se règlent ici.
fn arrival_section(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let until = match config.slots_until.trim() {
        "" => DEFAULT_SLOTS_UNTIL,
        until => until,
    };
    let late_from = config.late_from.as_deref().unwrap_or(DEFAULT_LATE_FROM);
    Section::new()
        .title("i18n:host.section.arrival")
        .child(
            named(config, ctx, "slots_until")
                .label("i18n:host.slots.until")
                .child(TimePicker::new().name("slots_until").value(until))
                .child(FieldHint::new().text("i18n:host.slots.step.hint")),
        )
        .child(
            Field::new()
                .name("slot_step")
                .label("i18n:host.slots.step")
                .child(segmented(
                    "slot_step",
                    config.step_shown(),
                    &["15", "30", "60"],
                    "host.slots.step",
                )),
        )
        .child(
            named(config, ctx, "late_from")
                .label("i18n:host.lateFrom")
                .child(TimePicker::new().name("late_from").value(late_from))
                .child(FieldHint::new().text("i18n:host.lateFrom.hint")),
        )
        .into()
}

// ── 2.2 Questions ────────────────────────────────────────────────────────────

fn questions_section(config: &ModuleConfig, ctx: &HostContext) -> Component {
    Section::new()
        .title("i18n:host.section.questions")
        .child(toggle(
            "ask_transport",
            "host.question.transport",
            config.ask_transport,
        ))
        .child(toggle(
            "ask_occasion",
            "host.question.occasion",
            config.ask_occasion,
        ))
        .child(toggle(
            "ask_needs",
            "host.question.needs",
            config.asks_needs(),
        ))
        .child(toggle(
            "ask_guest_count",
            "host.question.guestCount",
            config.ask_guest_count,
        ))
        .child(custom_questions(config, ctx))
        .into()
}

fn toggle(name: &str, key: &str, checked: bool) -> ToggleRow {
    ToggleRow::new()
        .name(name)
        .label(format!("i18n:{key}"))
        .description(format!("i18n:{key}.hint"))
        .checked(checked)
}

/// « Vos questions » : le tableau ordonnable, « N / 5 », et la fenêtre de chaque question.
fn custom_questions(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let rows = rows(config, ctx);
    let mut children: Vec<Component> = vec![FieldHint::new().text("i18n:host.custom.help").into()];
    if let Some(error) = config.error_of("custom_questions") {
        children.push(
            InlineNotice::new()
                .tone(Tone::Danger)
                .message(error.get(&ctx.locale).to_string())
                .into(),
        );
    }
    let mut table = DataTable::new()
        .rowName("custom_questions")
        .sortable(true)
        .panel("dialog")
        .columns(json!([
            { "key": "label", "label": "i18n:host.custom.label", "sub": "sub" },
            { "key": "kind", "label": "i18n:host.custom.type", "kind": "pill" },
        ]))
        .rows(Value::Array(
            rows.iter()
                .map(|(id, question)| row(id, question, ctx))
                .collect(),
        ))
        .children(
            rows.iter()
                .enumerate()
                .map(|(index, (id, question))| question_panel(index, id, question, config, ctx))
                .collect(),
        );
    if rows.len() < MAX_CUSTOM_QUESTIONS {
        let mut added: Vec<String> = rows
            .iter()
            .skip(config.custom_questions.len())
            .map(|(id, _)| id.clone())
            .collect();
        added.push(Uuid::new_v4().to_string());
        let mut input = Map::new();
        input.insert(NEW_QUESTIONS.into(), Value::String(added.join(",")));
        table = table
            .addLabel("i18n:host.custom.add")
            .addAction(Action::emit(
                contracts::shell::SURFACE_INPUT,
                Some(Value::Object(input)),
            ));
    }
    children.push(table.into());
    Section::new()
        .title("i18n:host.custom.title")
        .subtitle(format!("{} / {MAX_CUSTOM_QUESTIONS}", rows.len()))
        .children(children)
        .into()
}

/// Les lignes : les questions enregistrées, puis celles qu'« Ajouter » a ouvertes. Une question
/// sans id (enregistrée avant le tableau) en reçoit un, stable d'un rendu à l'autre
/// (`question-<rang>`) : la plateforme la rapproche par sa position, et l'id reste.
fn rows(config: &ModuleConfig, ctx: &HostContext) -> Vec<(String, CustomQuestion)> {
    let mut rows: Vec<(String, CustomQuestion)> = config
        .custom_questions
        .iter()
        .enumerate()
        .map(|(index, question)| {
            let id = match question.id.trim() {
                "" => format!("question-{index}"),
                id => id.to_string(),
            };
            (id, question.clone())
        })
        .collect();
    for id in ctx.input_str(NEW_QUESTIONS).unwrap_or_default().split(',') {
        let id = id.trim();
        if !id.is_empty()
            && rows.len() < MAX_CUSTOM_QUESTIONS
            && !rows.iter().any(|(known, _)| known == id)
        {
            rows.push((id.to_string(), CustomQuestion::default()));
        }
    }
    rows
}

/// Une ligne du tableau : la question, « Obligatoire » et les options dessous, la réponse en
/// pastille.
fn row(id: &str, question: &CustomQuestion, ctx: &HostContext) -> Value {
    let mut sub: Vec<String> = Vec::new();
    if question.required {
        sub.push(t!("host.custom.required").unwrap_or_default());
    }
    if question.kind == QuestionKind::Choice {
        sub.extend(
            question
                .options()
                .into_iter()
                .map(|o| o.host_value(ctx).to_string()),
        );
    }
    json!({
        "id": id,
        "label": question.label.host_value(ctx),
        "sub": sub.join(" · "),
        "kind": format!("i18n:host.custom.type.{}", question.kind.as_wire()),
    })
}

/// La fenêtre d'une question : Question, Réponse, Options (seulement pour « Choix »), Obligatoire.
fn question_panel(
    index: usize,
    id: &str,
    question: &CustomQuestion,
    config: &ModuleConfig,
    ctx: &HostContext,
) -> Component {
    let key = |name: &str| format!("custom_questions.{index}.{name}");
    // Les erreurs d'une question enregistrée ; une ligne neuve n'en a pas encore.
    let field = |name: &str| {
        let field = Field::new().name(key(name));
        match config
            .custom_questions
            .get(index)
            .and_then(|_| config.error_of(&key(name)))
        {
            Some(error) => field.error(error.get(&ctx.locale).to_string()),
            None => field,
        }
    };
    // Six cases : la fenêtre est rendue une fois, elle ne peut pas en ajouter. Vides, elles ne
    // comptent pas (`CustomQuestion::options`).
    let options: Vec<Component> = (0..MAX_OPTIONS)
        .map(|option| {
            TextInput::new()
                .name(format!("custom_questions.{index}.options.{option}.label"))
                .value(
                    question
                        .options
                        .get(option)
                        .map(|o| o.label.host_value(ctx))
                        .unwrap_or_default(),
                )
                .into()
        })
        .collect();
    Stack::new()
        .gap(14.0)
        .children(vec![
            sdui::row_id("custom_questions", index, Some(id)),
            field("label")
                .label("i18n:host.custom.label")
                .required(true)
                .child(
                    TextInput::new()
                        .name(key("label"))
                        .value(question.label.host_value(ctx))
                        .placeholder("i18n:host.custom.label.placeholder"),
                )
                .into(),
            Field::new()
                .name(key("type"))
                .label("i18n:host.custom.type")
                .child(segmented(
                    &key("type"),
                    question.kind.as_wire(),
                    &QuestionKind::WIRE_VALUES,
                    "host.custom.type",
                ))
                .into(),
            field("options")
                .label("i18n:host.custom.options")
                .visibleWhen(format!("{}=choice", key("type")))
                .child(Stack::new().gap(6.0).children(options))
                .child(FieldHint::new().text("i18n:host.custom.options.hint"))
                .into(),
            ToggleRow::new()
                .name(key("required"))
                .label("i18n:host.custom.required")
                .checked(question.required)
                .into(),
        ])
        .into()
}

// ── 2.3 Envoi et relances ────────────────────────────────────────────────────

fn sending_section(config: &ModuleConfig) -> Component {
    Section::new()
        .title("i18n:host.section.reminder")
        .child(
            Field::new()
                .name("deadline")
                .label("i18n:host.deadline")
                .child(segmented(
                    "deadline",
                    config.deadline.as_wire(),
                    Deadline::CHOICE_LIST_WIRE_VALUES,
                    "host.deadline",
                )),
        )
        .child(
            ToggleRow::new()
                .name("reminder")
                .label("i18n:host.reminder")
                .description("i18n:host.reminder.hint")
                .checked(config.reminder),
        )
        // Toujours lu par le livret (`show_when.rs`) : il garde sa place, ici, sous un nom clair.
        .child(
            Field::new()
                .name("show_when")
                .label("i18n:host.when")
                .child(
                    ChoiceList::new()
                        .name("show_when")
                        .layout(ChoiceListLayout::Segmented)
                        .value(config.show_when.as_wire())
                        .choices(
                            ShowWhen::CHOICE_LIST_WIRE_VALUES
                                .iter()
                                .map(|key| {
                                    ChoiceOption::new(*key, format!("i18n:host.when.{key}.desc"))
                                })
                                .collect(),
                        ),
                ),
        )
        .into()
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Un choix en segments, libellés `i18n:{prefix}.{valeur}`.
fn segmented(name: &str, value: &str, values: &[&str], prefix: &str) -> ChoiceList {
    ChoiceList::new()
        .name(name)
        .layout(ChoiceListLayout::Segmented)
        .value(value)
        .choices(
            values
                .iter()
                .map(|key| ChoiceOption::new(*key, format!("i18n:{prefix}.{key}")))
                .collect(),
        )
}

/// Le champ `name`, avec le message de [`ModuleConfig::error_of`] sous lui s'il y en a un.
fn named(config: &ModuleConfig, ctx: &HostContext, name: &str) -> Field {
    let field = Field::new().name(name);
    match config.error_of(name) {
        Some(error) => field.error(error.get(&ctx.locale).to_string()),
        None => field,
    }
}
