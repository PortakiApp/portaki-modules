//! Host dashboard surface — le délai de signalement et les options de restitution.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{
    Card, Field, FieldHint, Form, NumberInput, Page, Select, Stack, TextInput, Toggle,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{ModuleConfig, MAX_WINDOW_DAYS, MIN_WINDOW_DAYS};

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    label_key = "catalog.host.main",
    icon = IconName::Search
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;

    Ok(Surface::new(
        Page::new().child(
            Form::new().child(
                Stack::new()
                    .gap(16.0)
                    .child(window_card(&config, &ctx))
                    .child(return_card(&config)),
            ),
        ),
    )
    .with_id(MAIN))
}

/// Le délai : combien de jours après le départ le voyageur peut encore signaler.
fn window_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    Card::new()
        .title("i18n:host.window.title")
        .subtitle("i18n:host.window.subtitle")
        .icon(IconName::ClockCircle)
        .child(
            Field::new()
                .name("window_days")
                .label("i18n:host.window.label")
                .child(
                    NumberInput::new()
                        .name("window_days")
                        .min(f64::from(MIN_WINDOW_DAYS))
                        .max(f64::from(MAX_WINDOW_DAYS))
                        .value(f64::from(config.window_days())),
                ),
        )
        .child(FieldHint::new().text("i18n:host.window.hint"))
        // La promesse à côté du délai : les deux répondent à « et après ? », l'une pour le
        // voyageur qui hésite à déclarer, l'autre pour celui qui a déclaré.
        .child(
            Field::new()
                .name("response_delay")
                .label("i18n:host.responseDelay.label")
                .child(
                    TextInput::new()
                        .name("response_delay")
                        .value(config.response_delay.host_value(ctx))
                        .placeholder("i18n:host.responseDelay.placeholder"),
                ),
        )
        .child(FieldHint::new().text("i18n:host.responseDelay.hint"))
        .into()
}

/// Les options de restitution, et qui paie le renvoi.
fn return_card(config: &ModuleConfig) -> Component {
    let mut card = Card::new()
        .title("i18n:host.return.title")
        .subtitle("i18n:host.return.subtitle")
        .icon(IconName::Package);

    let offered = config.return_options();
    for (name, key) in [
        ("return_ship", "ship"),
        ("return_pickup", "pickup"),
        ("return_donate", "donate"),
    ] {
        card = card.child(
            Field::new()
                .name(name)
                .label(format!("i18n:host.return.{key}"))
                .child(Toggle::new().name(name).checked(offered.contains(&key))),
        );
    }

    // Qui paie n'a de sens que si le renvoi est proposé — sinon c'est une question sans objet.
    if config.offers_shipping() {
        card = card
            .child(
                Field::new()
                    .name("shipping_paid_by")
                    .label("i18n:host.shipping.label")
                    .child(
                        Select::new()
                            .name("shipping_paid_by")
                            .options(vec![
                                ChoiceOption::new("guest", "i18n:host.shipping.label.guest"),
                                ChoiceOption::new("host", "i18n:host.shipping.label.host"),
                            ])
                            .value(if config.shipping_paid_by_guest() {
                                "guest"
                            } else {
                                "host"
                            }),
                    ),
            )
            .child(FieldHint::new().text("i18n:host.shipping.hint"));
    }

    card.into()
}
