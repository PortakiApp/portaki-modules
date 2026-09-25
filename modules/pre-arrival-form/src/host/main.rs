//! Host property surface — design `editorPrearrival` / `prearrival-editor-v1`.
//!
//! Choice cards for when to show + toggle grid for form questions.
//! Save chrome is owned by the workspace tab; the platform stores the declared config.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, ChoiceList, Form, Grid, Page, Stack, ToggleRow};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{ModuleConfig, ShowWhen};

/// Host main — editable pre-arrival timing + question toggles.
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::PrearrivalEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Clipboard
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::read(&ctx)?;

    let form_children: Vec<Component> = vec![
        Card::new()
            .title("i18n:host.section.when")
            .subtitle("i18n:host.section.when.help")
            .icon(IconName::ClockCircle)
            .children(vec![when_choice_list(config.show_when).into()])
            .into(),
        Card::new()
            .title("i18n:host.section.questions")
            .subtitle("i18n:host.section.questions.help")
            .icon(IconName::Clipboard)
            .children(vec![Grid::new()
                .columns(2)
                .gap(8.0)
                .minColumnWidth(300.0)
                .children(question_toggle_rows(&config))
                .into()])
            .into(),
    ];

    // No Page title / Save — workspace tab owns chrome + footer Save.
    Ok(Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(16.0).children(form_children))),
    )
    .with_id(crate::ids::HOST_MAIN))
}

fn when_choice_list(selected: ShowWhen) -> ChoiceList {
    ChoiceList::new()
        .name("show_when")
        .value(selected.as_wire())
        .layout(ChoiceListLayout::Cards)
        .choices(vec![
            ChoiceOption::new("confirm", "i18n:host.when.confirm")
                .description("i18n:host.when.confirm.desc")
                .icon(IconName::CheckCircle),
            ChoiceOption::new("before", "i18n:host.when.before")
                .description("i18n:host.when.before.desc")
                .icon(IconName::ClockCircle),
            ChoiceOption::new("checkin", "i18n:host.when.checkin")
                .description("i18n:host.when.checkin.desc")
                .icon(IconName::Key),
        ])
}

fn question_toggle_rows(questions: &ModuleConfig) -> Vec<Component> {
    vec![
        toggle_row(
            "ask_arrival_time",
            "i18n:host.question.arrival",
            IconName::ClockCircle,
            questions.ask_arrival_time,
        ),
        toggle_row(
            "ask_occasion",
            "i18n:host.question.occasion",
            IconName::Gift,
            questions.ask_occasion,
        ),
        toggle_row(
            "ask_allergies",
            "i18n:host.question.allergies",
            IconName::InfoCircle,
            questions.ask_allergies,
        ),
        toggle_row(
            "ask_guest_count",
            "i18n:host.question.guestCount",
            IconName::Users,
            questions.ask_guest_count,
        ),
        toggle_row(
            "ask_special_needs",
            "i18n:host.question.specialNeeds",
            IconName::Home,
            questions.ask_special_needs,
        ),
        toggle_row(
            "ask_id_document",
            "i18n:host.question.idDocument",
            IconName::Clipboard,
            questions.ask_id_document,
        ),
    ]
}

fn toggle_row(name: &str, label: &str, icon: IconName, checked: bool) -> Component {
    // Bordered tile + leading icon chip (design `editorPrearrival` question grid).
    ToggleRow::new()
        .name(name)
        .label(label)
        .icon(icon)
        .checked(checked)
        .into()
}
