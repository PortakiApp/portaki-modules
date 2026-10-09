//! Host dashboard surface — le tiroir de réglages (spec Signaler §2) : le formulaire, puis la
//! réception. Le chrome du tiroir (titre, interrupteur, Publier) est au dashboard.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{
    Card, Field, FieldHint, Form, Page, Select, Stack, TextInput, Toggle,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{ModuleConfig, PHASES, RESPONSE_TIMES};

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyModuleSheet,
    label_key = "catalog.host.main",
    icon = IconName::MessageCircle
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    Ok(Surface::new(
        Page::new().child(
            Form::new().child(
                Stack::new()
                    .gap(16.0)
                    .child(form_card(&config, &ctx))
                    .child(reception_card(&config, &ctx)),
            ),
        ),
    )
    .with_id(MAIN))
}

/// §2.1 Le formulaire : catégories, photo, urgence, périodes.
fn form_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let mut card = Card::new()
        .title("i18n:host.category.title")
        .subtitle("i18n:host.category.subtitle")
        .icon(IconName::MessageCircle);
    // Les cases telles qu'enregistrées, pas l'ensemble de repli : tout décoché reste décoché,
    // avec « Choisissez au moins une catégorie. » sous la dernière.
    let ticks = config.category_ticks();
    for (index, wire) in crate::category::WIRE_VALUES.iter().enumerate() {
        let field = toggle(
            &format!("category_{wire}"),
            &format!("i18n:host.category.{wire}"),
            ticks[index],
        );
        card = card.child(if index + 1 == ticks.len() {
            with_error(field, config, ctx, "categories")
        } else {
            field
        });
    }
    card = card
        .child(FieldHint::new().text("i18n:host.category.hint"))
        .child(toggle(
            "photo_allowed",
            "i18n:host.photo.label",
            config.photo_allowed(),
        ))
        .child(toggle(
            "urgent_phone",
            "i18n:host.urgent.label",
            config.urgent_phone(),
        ))
        .child(FieldHint::new().text("i18n:host.urgent.hint"));
    // Le numéro n'a d'objet que si l'option est ouverte (règles communes : masqué, pas grisé).
    if config.urgent_phone() {
        card = card
            .child(
                named(config, ctx, "urgent_number")
                    .label("i18n:host.urgentNumber.label")
                    .child(
                        TextInput::new()
                            .name("urgent_number")
                            .value(config.urgent_number.clone())
                            .placeholder("+33 6 12 34 56 78"),
                    ),
            )
            .child(FieldHint::new().text("i18n:host.urgentNumber.hint"));
    }
    let ticks = config.phase_ticks();
    for (index, phase) in PHASES.iter().enumerate() {
        let field = toggle(
            &format!("phase_{phase}"),
            &format!("i18n:host.phase.{phase}"),
            ticks[index],
        );
        card = card.child(if index + 1 == PHASES.len() {
            with_error(field, config, ctx, "phases")
        } else {
            field
        });
    }
    card.child(FieldHint::new().text("i18n:host.phase.hint"))
        .into()
}

/// §2.2 La réception : la confirmation et le délai annoncé.
fn reception_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    Card::new()
        .title("i18n:host.reception.title")
        .icon(IconName::Bell)
        .child(
            named(config, ctx, "auto_reply")
                .label("i18n:host.autoReply.label")
                .child(
                    TextInput::new()
                        .name("auto_reply")
                        .value(config.auto_reply.host_value(ctx))
                        .placeholder("i18n:host.autoReply.placeholder"),
                ),
        )
        .child(
            Field::new()
                .name("response_time")
                .label("i18n:host.responseTime.label")
                .child(
                    Select::new()
                        .name("response_time")
                        .options(
                            RESPONSE_TIMES
                                .iter()
                                .map(|key| {
                                    ChoiceOption::new(
                                        *key,
                                        format!("i18n:host.responseTime.label.{key}"),
                                    )
                                })
                                .collect(),
                        )
                        .value(config.response_time()),
                ),
        )
        .into()
}

fn toggle(name: &str, label: &str, checked: bool) -> Field {
    Field::new()
        .name(name)
        .label(label)
        .child(Toggle::new().name(name).checked(checked))
}

/// Le champ `name`, avec le message de [`ModuleConfig::error_of`] sous lui s'il y en a un.
fn named(config: &ModuleConfig, ctx: &HostContext, name: &str) -> Field {
    with_error(Field::new().name(name), config, ctx, name)
}

/// `field`, avec le message de [`ModuleConfig::error_of`] pour `problem` sous lui s'il y en a un.
fn with_error(field: Field, config: &ModuleConfig, ctx: &HostContext, problem: &str) -> Field {
    match config.error_of(problem) {
        Some(error) => field.error(error.get(&ctx.locale).to_string()),
        None => field,
    }
}
