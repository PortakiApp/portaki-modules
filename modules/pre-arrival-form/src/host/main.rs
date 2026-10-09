//! Host property surface — design `editorPrearrival` / `prearrival-editor-v1`.
//!
//! Choice cards for when to show + toggle grid for form questions.
//! Save chrome is owned by the workspace tab; the platform stores the declared config.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{
    Card, ChoiceList, Field, FieldHint, Form, Grid, Page, Select, Stack, TextInput, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{Deadline, ModuleConfig, ShowWhen, DEFAULT_SLOTS_UNTIL};

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
    let config = ModuleConfig::load(&ctx)?;

    let mut form_children: Vec<Component> = vec![
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
        reminder_card(&config),
    ];

    // Les créneaux n'ont d'objet que si l'heure d'arrivée est demandée (règles communes).
    if config.ask_arrival_time {
        form_children.insert(1, slots_card(&config, &ctx));
    }

    // No Page title / Save — workspace tab owns chrome + footer Save.
    Ok(Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(16.0).children(form_children))),
    )
    .with_id(MAIN))
}

/// §2.1 Les créneaux d'arrivée : les trois plages, ou un pas régulier jusqu'à une heure de fin.
fn slots_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let step = match config.slot_step.trim() {
        "15" | "30" | "60" => config.slot_step.trim(),
        _ => "ranges",
    };
    let mut card = Card::new()
        .title("i18n:host.slots.title")
        .icon(IconName::ClockCircle)
        .child(
            Field::new()
                .name("slot_step")
                .label("i18n:host.slots.step")
                .child(
                    Select::new()
                        .name("slot_step")
                        .options(
                            ["ranges", "15", "30", "60"]
                                .into_iter()
                                .map(|key| {
                                    ChoiceOption::new(key, format!("i18n:host.slots.step.{key}"))
                                })
                                .collect(),
                        )
                        .value(step),
                ),
        )
        .child(FieldHint::new().text("i18n:host.slots.step.hint"));
    if step != "ranges" {
        let mut until = Field::new()
            .name("slots_until")
            .label("i18n:host.slots.until");
        if let Some((_, error)) = config.problems().into_iter().next() {
            until = until.error(error.get(&ctx.locale).to_string());
        }
        card = card.child(
            until.child(
                TextInput::new()
                    .name("slots_until")
                    .value(config.slots_until.clone())
                    .placeholder(DEFAULT_SLOTS_UNTIL),
            ),
        );
    }
    card.into()
}

/// §2.3 Envoi et relances : la limite, et l'e-mail de la veille.
fn reminder_card(config: &ModuleConfig) -> Component {
    Card::new()
        .title("i18n:host.section.reminder")
        .icon(IconName::Bell)
        .child(
            Field::new()
                .name("deadline")
                .label("i18n:host.deadline")
                .child(
                    Select::new()
                        .name("deadline")
                        .options(
                            Deadline::CHOICE_LIST_WIRE_VALUES
                                .iter()
                                .map(|key| {
                                    ChoiceOption::new(*key, format!("i18n:host.deadline.{key}"))
                                })
                                .collect(),
                        )
                        .value(config.deadline.as_wire()),
                ),
        )
        .child(
            ToggleRow::new()
                .name("reminder")
                .label("i18n:host.reminder")
                .description("i18n:host.reminder.hint")
                .checked(config.reminder),
        )
        .into()
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
        // Juste après l'heure : les deux disent ce que l'hôte doit préparer — une place de
        // parking, un horaire de train, un transfert.
        toggle_row(
            "ask_transport",
            "i18n:host.question.transport",
            IconName::Car,
            questions.ask_transport,
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
