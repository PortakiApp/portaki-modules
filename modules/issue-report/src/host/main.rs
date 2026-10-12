//! Host dashboard surface — le tiroir de réglages (spec Signaler §2) : le formulaire, puis la
//! réception. Le chrome du tiroir (titre, interrupteur, Publier) est au dashboard.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Card, ChoiceList, Field, FieldHint, Form, InlineNotice, Page, PhoneInput, Pill, Stack,
    TextArea, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{ModuleConfig, PHASES, RESPONSE_TIMES};

#[portaki_sdk::nav(
    placement = HostPlacement::WorkspaceTimelineTask,
    path = "tasks",
    label_key = "catalog.host.tasks",
    icon = IconName::DangerTriangle
)]
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyModuleSheet,
    label_key = "catalog.host.main",
    icon = IconName::MessageCircle
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    let mut page = Stack::new().gap(16.0);
    if let Some(note) = open_reports_note() {
        page = page.child(note);
    }
    Ok(Surface::new(
        Page::new().child(
            Form::new().child(
                page.child(form_card(&config, &ctx))
                    .child(reception_card(&config, &ctx)),
            ),
        ),
    )
    .with_id(MAIN))
}

/// « 2 signalements en cours sur ce logement. » en tête du tiroir ; rien quand tout est résolu.
fn open_reports_note() -> Option<Component> {
    let epoch = chrono::DateTime::<chrono::Utc>::UNIX_EPOCH;
    let open = crate::storage::list_since(epoch)
        .unwrap_or_default()
        .iter()
        .filter(|report| report.resolved_at.is_none())
        .count();
    let message = match open {
        0 => return None,
        1 => t!("host.open.one"),
        count => t!("host.open.many", count = count),
    };
    Some(
        InlineNotice::new()
            .tone(Tone::Neutral)
            .message(message.unwrap_or_default())
            .into(),
    )
}

/// §2.1 Le formulaire : catégories, périodes, photo, urgence.
fn form_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    // L'ensemble tel qu'enregistré, pas celui de repli : tout décoché reste décoché, avec
    // « Choisissez au moins une catégorie. » sous le champ.
    let categories = config.chosen_categories();
    let mut category_field = named(config, ctx, "categories")
        .label("i18n:host.category.title")
        .child(multi_choice(
            "categories",
            crate::category::WIRE_VALUES
                .iter()
                .map(|wire| ChoiceOption::new(*wire, format!("i18n:host.category.{wire}")))
                .collect(),
            &categories,
        ));
    if categories.len() == 1 {
        category_field = category_field.child(FieldHint::new().text("i18n:host.category.one"));
    }
    Card::new()
        .title("i18n:host.form.title")
        .child(category_field)
        .child(
            named(config, ctx, "phases")
                .label("i18n:host.phase.title")
                .child(multi_choice(
                    "phases",
                    PHASES
                        .iter()
                        .map(|phase| ChoiceOption::new(*phase, format!("i18n:host.phase.{phase}")))
                        .collect(),
                    &config.chosen_phases(),
                ))
                .child(FieldHint::new().text("i18n:host.phase.hint")),
        )
        .child(
            ToggleRow::new()
                .name("photo_allowed")
                .label("i18n:host.photo.label")
                .description("i18n:host.photo.help")
                .checked(config.photo_allowed()),
        )
        .child(
            ToggleRow::new()
                .name("urgent_phone")
                .label("i18n:host.urgent.label")
                .description("i18n:host.urgent.hint")
                .checked(config.urgent_phone()),
        )
        // Le numéro n'a d'objet que si l'option est ouverte (règles communes : masqué, pas grisé).
        .child(
            named(config, ctx, "urgent_number")
                .label("i18n:host.urgentNumber.label")
                .visibleWhen("urgent_phone=true")
                .child(
                    PhoneInput::new()
                        .name("urgent_number")
                        .value(crate::config::compact(&config.urgent_number)),
                )
                .child(FieldHint::new().text("i18n:host.urgentNumber.hint")),
        )
        .child(
            Field::new().visibleWhen("urgent_phone=false").child(
                InlineNotice::new()
                    .tone(Tone::Neutral)
                    .message("i18n:host.urgent.off"),
            ),
        )
        .into()
}

/// §2.2 La réception : les canaux, la confirmation et le délai annoncé.
fn reception_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    // Le délai du brouillon d'abord : le choisir re-rend le tiroir, et l'aide suit.
    let response_time = ctx
        .input_str("response_time")
        .and_then(|raw| {
            RESPONSE_TIMES
                .iter()
                .copied()
                .find(|key| *key == raw.trim())
        })
        .unwrap_or_else(|| config.response_time());
    Card::new()
        .title("i18n:host.reception.title")
        // Seul l'e-mail part aujourd'hui : les deux autres canaux sont annoncés, pas proposés.
        .child(
            Field::new().label("i18n:host.notify.label").child(
                Stack::new()
                    .direction(StackDirection::Horizontal)
                    .gap(6.0)
                    .child(Pill::new().label("i18n:host.notify.email"))
                    .child(Pill::new().label("i18n:host.notify.app"))
                    .child(Pill::new().label("i18n:host.notify.sms")),
            ),
        )
        .child(
            named(config, ctx, "auto_reply")
                .label("i18n:host.autoReply.label")
                .child(
                    TextArea::new()
                        .name("auto_reply")
                        .rows(2)
                        .value(config.auto_reply.host_value(ctx))
                        .placeholder("i18n:host.autoReply.placeholder"),
                )
                .child(FieldHint::new().text("i18n:host.autoReply.hint")),
        )
        .child(
            Field::new()
                .name("response_time")
                .label("i18n:host.responseTime.label")
                .child(
                    ChoiceList::new()
                        .name("response_time")
                        .layout(ChoiceListLayout::Segmented)
                        .emitOnChange(true)
                        .choices(
                            ["within_hour", "within_day", "none"]
                                .iter()
                                .map(|key| {
                                    ChoiceOption::new(
                                        *key,
                                        format!("i18n:host.responseTime.label.{key}"),
                                    )
                                })
                                .collect(),
                        )
                        .value(response_time),
                )
                .child(
                    FieldHint::new().text(format!("i18n:host.responseTime.hint.{response_time}")),
                ),
        )
        .into()
}

/// Un choix multiple en pastilles ; la valeur est le tableau coché, en JSON.
fn multi_choice(name: &str, choices: Vec<ChoiceOption>, chosen: &[&str]) -> ChoiceList {
    ChoiceList::new()
        .name(name)
        .multi(true)
        .layout(ChoiceListLayout::Compact)
        .choices(choices)
        .value(serde_json::to_string(chosen).unwrap_or_default())
}

/// Le champ `name`, avec le message de [`ModuleConfig::error_of`] sous lui s'il y en a un.
fn named(config: &ModuleConfig, ctx: &HostContext, name: &str) -> Field {
    let field = Field::new().name(name);
    match config.error_of(name) {
        Some(error) => field.error(error.get(&ctx.locale).to_string()),
        None => field,
    }
}
