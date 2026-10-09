//! Host dashboard surface — design `editorEvParking` / `evparking-editor-v1`.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{
    Card, ChoiceList, Field, FieldHint, Form, NumberInput, Page, SecretInput, Select, Stack,
    TextArea, TextInput, Toggle,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{ModuleConfig, RevealPolicy, CHARGER_TYPES, POWER_KW, PRICINGS};

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::EvparkingEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Zap
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;

    let mut form_children: Vec<Component> = vec![
        Card::new()
            .title("i18n:host.section.spot")
            .subtitle("i18n:host.section.spot.help")
            .icon(IconName::Zap)
            .children(vec![
                named(&config, &ctx, "spot_label")
                    .label("i18n:host.spotLabel.label")
                    .required(true)
                    .child(
                        TextInput::new()
                            .name("spot_label")
                            .value(config.spot_label.host_value(&ctx))
                            .placeholder("i18n:host.spotLabel.placeholder"),
                    )
                    .into(),
                Field::new()
                    .name("parking_code")
                    .label("i18n:host.parkingCode.label")
                    .child(
                        SecretInput::new()
                            .name("parking_code")
                            .value(String::new())
                            .placeholder("i18n:host.parkingCode.placeholder"),
                    )
                    .into(),
                Field::new()
                    .name("charger_pin")
                    .label("i18n:host.chargerPin.label")
                    .child(
                        SecretInput::new()
                            .name("charger_pin")
                            .value(String::new())
                            .placeholder("i18n:host.chargerPin.placeholder"),
                    )
                    .into(),
                Field::new()
                    .name("map_url")
                    .label("i18n:host.mapUrl.label")
                    .child(
                        TextInput::new()
                            .name("map_url")
                            .value(config.map_url.clone().unwrap_or_default())
                            .placeholder("i18n:host.mapUrl.placeholder"),
                    )
                    .into(),
            ])
            .into(),
        Card::new()
            .title("i18n:host.section.instructions")
            .subtitle("i18n:host.section.instructions.help")
            .icon(IconName::InfoCircle)
            .children(vec![named(&config, &ctx, "instructions")
                .label("i18n:host.instructions.label")
                .child(
                    // TipTap preferred in design; TextArea until guest renders rich HTML.
                    TextArea::new()
                        .name("instructions")
                        .value(
                            config
                                .instructions
                                .as_ref()
                                .map(|text| text.host_value(&ctx))
                                .unwrap_or_default(),
                        )
                        .placeholder("i18n:host.instructions.placeholder"),
                )
                .into()])
            .into(),
        Card::new()
            .title("i18n:host.section.reveal")
            .subtitle("i18n:host.section.reveal.help")
            .icon(IconName::ClockCircle)
            .children(vec![reveal_choice_list(config.reveal_policy).into()])
            .into(),
    ];

    // La borne et le tarif après la place et ses codes (§2.2, §2.3).
    form_children.insert(1, charger_card(&config, &ctx));
    form_children.insert(2, pricing_card(&config, &ctx));

    // No Page title / Save — the modules sheet owns chrome + footer Save.
    Ok(Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(16.0).children(form_children))),
    )
    .with_id(MAIN))
}

/// §2.2 La borne : la prise, la puissance, le câble.
fn charger_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let mut power = NumberInput::new()
        .name("power_kw")
        .min(POWER_KW.0)
        .max(POWER_KW.1);
    if config.power_kw != 0.0 {
        power = power.value(config.power_kw);
    }
    Card::new()
        .title("i18n:host.section.charger")
        .icon(IconName::Zap)
        .child(select(
            "charger_type",
            "i18n:host.chargerType.label",
            &CHARGER_TYPES,
            config.charger_type(),
        ))
        .child(
            named(config, ctx, "power_kw")
                .label("i18n:host.power.label")
                .child(power),
        )
        .child(FieldHint::new().text("i18n:host.power.hint"))
        .child(
            Field::new()
                .name("cable_provided")
                .label("i18n:host.cable.label")
                .child(
                    Toggle::new()
                        .name("cable_provided")
                        .checked(config.cable_provided()),
                ),
        )
        .into()
}

/// §2.3 Tarif et réservation : le prix n'est demandé que si la recharge n'est pas incluse, la
/// consigne de réservation que si elle est requise (règles communes : masqué, pas grisé).
fn pricing_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let mut card = Card::new()
        .title("i18n:host.section.pricing")
        .icon(IconName::Ticket)
        .child(select(
            "pricing",
            "i18n:host.pricing.label",
            &PRICINGS,
            config.pricing(),
        ));
    if config.pricing() != "included" {
        card = card.child(
            named(config, ctx, "price")
                .label("i18n:host.price.label")
                .required(true)
                .child(
                    TextInput::new()
                        .name("price")
                        .value(config.price.clone())
                        .placeholder("i18n:host.price.placeholder"),
                ),
        );
    }
    card = card
        .child(
            Field::new()
                .name("booking_required")
                .label("i18n:host.booking.label")
                .child(
                    Toggle::new()
                        .name("booking_required")
                        .checked(config.booking_required),
                ),
        )
        .child(FieldHint::new().text("i18n:host.booking.hint"));
    if config.booking_required {
        card = card.child(
            named(config, ctx, "booking_note")
                .label("i18n:host.bookingNote.label")
                .child(
                    TextInput::new()
                        .name("booking_note")
                        .value(
                            config
                                .booking_note
                                .as_ref()
                                .map(|note| note.host_value(ctx))
                                .unwrap_or_default(),
                        )
                        .placeholder("i18n:host.bookingNote.placeholder"),
                ),
        );
    }
    card.into()
}

fn select(name: &str, label: &str, values: &[&str], chosen: &str) -> Field {
    Field::new().name(name).label(label).child(
        Select::new()
            .name(name)
            .options(
                values
                    .iter()
                    .map(|value| ChoiceOption::new(*value, format!("{label}.{value}")))
                    .collect(),
            )
            .value(chosen),
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

fn reveal_choice_list(policy: RevealPolicy) -> ChoiceList {
    ChoiceList::new()
        .name("reveal_policy")
        .value(policy.as_wire())
        .choices(vec![
            ChoiceOption::new(RevealPolicy::Always.as_wire(), "i18n:host.reveal.always")
                .description("i18n:host.reveal.always.desc")
                .icon(IconName::ClockCircle),
            ChoiceOption::new(
                RevealPolicy::HoursBefore24.as_wire(),
                "i18n:host.reveal.hoursBefore24",
            )
            .description("i18n:host.reveal.hoursBefore24.desc")
            .icon(IconName::ClockCircle),
            ChoiceOption::new(
                RevealPolicy::DayBefore16h.as_wire(),
                "i18n:host.reveal.dayBefore16h",
            )
            .description("i18n:host.reveal.dayBefore16h.desc")
            .icon(IconName::ClockCircle),
            ChoiceOption::new(
                RevealPolicy::AtCheckin.as_wire(),
                "i18n:host.reveal.atCheckin",
            )
            .description("i18n:host.reveal.atCheckin.desc")
            .icon(IconName::Key),
        ])
}
