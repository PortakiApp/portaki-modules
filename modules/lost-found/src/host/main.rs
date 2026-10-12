//! Host dashboard surface — le tiroir de réglages (spec Objet oublié §2) : le délai et la garde,
//! puis les options de restitution. Le chrome du tiroir (titre, interrupteur, Publier) est au
//! dashboard.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{ChoiceListLayout, Tone};
use portaki_sdk::sdui::primitives::{
    Card, ChoiceList, Field, FieldHint, Form, InlineNotice, NumberInput, Page, Stack, TextArea,
    TextInput, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{ModuleConfig, MAX_KEEP_DAYS, MAX_WINDOW_DAYS, MIN_KEEP_DAYS, MIN_WINDOW_DAYS};

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyModuleSheet,
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
                    .child(return_card(&config, &ctx)),
            ),
        ),
    )
    .with_id(MAIN))
}

/// Le délai : combien de jours après le départ le voyageur peut encore signaler.
fn window_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    Card::new()
        .title("i18n:host.window.title")
        .icon(IconName::ClockCircle)
        .child(with_error(
            Field::new()
                .name("window_days")
                .label("i18n:host.window.label")
                .child(
                    NumberInput::new()
                        .name("window_days")
                        .min(f64::from(MIN_WINDOW_DAYS))
                        .max(f64::from(MAX_WINDOW_DAYS))
                        .value(shown(config.window_days, config.window_days())),
                ),
            config,
            ctx,
        ))
        .child(FieldHint::new().text("i18n:host.window.hint"))
        .child(with_error(
            Field::new()
                .name("keep_days")
                .label("i18n:host.keep.label")
                .child(
                    NumberInput::new()
                        .name("keep_days")
                        .min(f64::from(MIN_KEEP_DAYS))
                        .max(f64::from(MAX_KEEP_DAYS))
                        .value(shown(config.keep_days, config.keep_days())),
                ),
            config,
            ctx,
        ))
        .child(FieldHint::new().text("i18n:host.keep.hint"))
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

/// Les options de restitution, chacune suivie de ce qu'elle demande : qui paie le renvoi, où
/// récupérer, quelle association. Un réglage dépendant suit l'interrupteur du brouillon
/// (`visibleWhen`, masqué et non grisé) ; masqué, il n'est pas envoyé et sa valeur reste.
fn return_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let chosen = config.chosen_options();
    let toggle = |name: &str, key: &str| -> Component {
        ToggleRow::new()
            .name(name)
            .label(format!("i18n:host.return.{key}"))
            .checked(chosen.contains(&key))
            .into()
    };
    let mut card = Card::new()
        .title("i18n:host.return.title")
        .subtitle("i18n:host.return.subtitle")
        .icon(IconName::Package);
    if let Some(error) = config.error_of("return") {
        card = card.child(
            InlineNotice::new()
                .tone(Tone::Danger)
                .message(error.get(&ctx.locale).to_string()),
        );
    }
    card.child(toggle("return_ship", "ship"))
        .child(
            Field::new()
                .name("shipping_paid_by")
                .label("i18n:host.shipping.label")
                .visibleWhen("return_ship=true")
                .child(
                    ChoiceList::new()
                        .name("shipping_paid_by")
                        .layout(ChoiceListLayout::Segmented)
                        .value(if config.shipping_paid_by_guest() {
                            "guest"
                        } else {
                            "host"
                        })
                        .choices(vec![
                            ChoiceOption::new("guest", "i18n:host.shipping.label.guest"),
                            ChoiceOption::new("host", "i18n:host.shipping.label.host"),
                        ]),
                ),
        )
        .child(toggle("return_pickup", "pickup"))
        .child(with_error(
            Field::new()
                .name("pickup_note")
                .label("i18n:host.pickupNote.label")
                .visibleWhen("return_pickup=true")
                .child(
                    TextArea::new()
                        .name("pickup_note")
                        .value(config.pickup_note.host_value(ctx))
                        .placeholder("i18n:host.pickupNote.placeholder"),
                ),
            config,
            ctx,
        ))
        .child(toggle("return_donate", "donate"))
        .child(with_error(
            Field::new()
                .name("donate_org")
                .label("i18n:host.donateOrg.label")
                .required(true)
                .visibleWhen("return_donate=true")
                .child(
                    TextInput::new()
                        .name("donate_org")
                        .value(config.donate_org.clone())
                        .placeholder("i18n:host.donateOrg.placeholder"),
                ),
            config,
            ctx,
        ))
        .into()
}

/// La valeur à montrer : celle que l'hôte a saisie, même hors bornes — c'est elle que l'erreur
/// sous le champ désigne. L'effective seulement quand il n'a rien choisi.
fn shown(stored: f64, effective: u32) -> f64 {
    if stored.is_finite() && stored != 0.0 {
        stored
    } else {
        f64::from(effective)
    }
}

/// Le champ, avec le message de [`ModuleConfig::error_of`] sous lui s'il y en a un.
fn with_error(field: Field, config: &ModuleConfig, ctx: &HostContext) -> Field {
    let name = field.name.clone().unwrap_or_default();
    match config.error_of(&name) {
        Some(error) => field.error(error.get(&ctx.locale).to_string()),
        None => field,
    }
}
