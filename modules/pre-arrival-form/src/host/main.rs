//! Host property surface — design `editorPrearrival` / `prearrival-editor-v1`.
//!
//! Choice cards for when to show + toggle grid for form questions.
//! Save chrome is owned by the workspace tab (`updateConfig`).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, ChoiceList, Form, Grid, Page, Stack, ToggleRow};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{load_config, FormQuestions, ShowWhen};

/// Host main — editable pre-arrival timing + question toggles.
#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(_ctx: HostContext) -> Surface {
    let config = load_config().unwrap_or_default();
    let questions = &config.questions;

    let form_children: Vec<Component> = vec![
        Card::new()
            .title("i18n:host.section.when")
            .subtitle("i18n:host.section.when.help")
            .icon("clock-circle")
            .children(vec![when_choice_list(config.show_when).into()])
            .into(),
        Card::new()
            .title("i18n:host.section.questions")
            .subtitle("i18n:host.section.questions.help")
            .icon("clipboard")
            .children(vec![Grid::new()
                .columns(2)
                .gap(8.0)
                .minColumnWidth(300.0)
                .children(question_toggle_rows(questions))
                .into()])
            .into(),
    ];

    // No Page title / Save — workspace tab owns chrome + footer Save.
    Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(16.0).children(form_children))),
    )
    .with_id(crate::ids::HOST_MAIN)
}

fn when_choice_list(selected: ShowWhen) -> ChoiceList {
    ChoiceList::new()
        .name("show_when")
        .value(selected.as_wire())
        .layout(ChoiceListLayout::Cards)
        .choices(vec![
            ChoiceOption::new("confirm", "i18n:host.when.confirm")
                .description("i18n:host.when.confirm.desc")
                .icon("check-circle"),
            ChoiceOption::new("before", "i18n:host.when.before")
                .description("i18n:host.when.before.desc")
                .icon("clock-circle"),
            ChoiceOption::new("checkin", "i18n:host.when.checkin")
                .description("i18n:host.when.checkin.desc")
                .icon("key"),
        ])
}

fn question_toggle_rows(questions: &FormQuestions) -> Vec<Component> {
    vec![
        toggle_row(
            "ask_arrival_time",
            "i18n:host.question.arrival",
            "clock-circle",
            questions.ask_arrival_time,
        ),
        toggle_row(
            "ask_occasion",
            "i18n:host.question.occasion",
            "gift",
            questions.ask_occasion,
        ),
        toggle_row(
            "ask_allergies",
            "i18n:host.question.allergies",
            "info-circle",
            questions.ask_allergies,
        ),
        toggle_row(
            "ask_guest_count",
            "i18n:host.question.guestCount",
            "users",
            questions.ask_guest_count,
        ),
        toggle_row(
            "ask_special_needs",
            "i18n:host.question.specialNeeds",
            "home",
            questions.ask_special_needs,
        ),
        toggle_row(
            "ask_id_document",
            "i18n:host.question.idDocument",
            "clipboard",
            questions.ask_id_document,
        ),
    ]
}

fn toggle_row(name: &str, label: &str, icon: &str, checked: bool) -> Component {
    // Leading icons need SDK ToggleRow.icon (portaki-sdk#27). Wire `.icon(icon)` after merge.
    let row = ToggleRow::new().name(name).label(label).checked(checked);
    let _ = icon;
    row.into()
}
