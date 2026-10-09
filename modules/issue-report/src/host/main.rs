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
    let offered = config.categories();
    let mut card = Card::new()
        .title("i18n:host.category.title")
        .subtitle("i18n:host.category.subtitle")
        .icon(IconName::MessageCircle);
    for wire in crate::category::WIRE_VALUES {
        card = card.child(toggle(
            &format!("category_{wire}"),
            &format!("i18n:host.category.{wire}"),
            offered.contains(wire),
        ));
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
    let phases = config.phases();
    for phase in PHASES {
        card = card.child(toggle(
            &format!("phase_{phase}"),
            &format!("i18n:host.phase.{phase}"),
            phases.contains(&phase),
        ));
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
    let field = Field::new().name(name);
    match config.error_of(name) {
        Some(error) => field.error(error.get(&ctx.locale).to_string()),
        None => field,
    }
}
