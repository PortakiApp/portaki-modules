//! Host dashboard surface — the settings page (spec Parking VE §2).
//!
//! Three `Section`s, the shell's three cards, in the spec's order: the spot, the charger, the
//! price and booking. The page chrome (title, switch, Publier) belongs to the dashboard. A rule
//! broken shows under its field (`Field::error`), the same one `publishReadiness` blocks on.
//!
//! A field that depends on a switch hides with `visibleWhen`. The price depends on three values
//! of four, which `visibleWhen` cannot say: the tariff list re-renders the page as it is chosen
//! (`emitOnChange`) and the draft's choice is read first.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    AddressMapPicker, ChoiceList, Field, FieldHint, Form, Grid, ImageUpload, InlineNotice,
    NumberInput, Page, RadioGroup, SecretInput, Section, SelectableCard, TextArea, TextInput,
    ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{charge_hours, ModuleConfig, RevealPolicy, CHARGER_TYPES, POWER_KW, PRICINGS};

mod stay;
pub use stay::render_host_stay;

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
    // Le brouillon d'abord : choisir un tarif re-rend la page avec ce choix.
    let pricing = ctx
        .input_str("pricing")
        .and_then(|raw| PRICINGS.iter().copied().find(|p| *p == raw.trim()))
        .unwrap_or_else(|| config.pricing());

    // No Save button — the shell saves the form (`updateConfig`, taken by the platform).
    Ok(Surface::new(
        Page::new().child(
            Form::new()
                .child(spot_section(&config, &ctx))
                .child(charger_section(&config, &ctx))
                .child(pricing_section(&config, &ctx, pricing)),
        ),
    )
    .with_id(MAIN))
}

// ── 2.1 La place ─────────────────────────────────────────────────────────────

fn spot_section(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let mut children: Vec<Component> = vec![field(
        config,
        ctx,
        "spot_label",
        "i18n:host.spotLabel.label",
        TextInput::new()
            .name("spot_label")
            .value(config.spot_label.host_value(ctx))
            .placeholder("i18n:host.spotLabel.placeholder"),
        None,
    )
    .required(true)
    .into()];
    children.push(position_picker(config, ctx).into());
    if config.far_from(ctx.property.coordinates.map(|c| (c.lat, c.lng))) {
        children.push(
            InlineNotice::new()
                .message("i18n:host.position.far")
                .tone(Tone::Warning)
                .into(),
        );
    }
    // Le lien carte d'avant l'épingle : gardé, lisible et effaçable, tant qu'il en reste un.
    if let Some(url) = config.map_url.as_deref().filter(|u| !u.trim().is_empty()) {
        children.push(
            field(
                config,
                ctx,
                "map_url",
                "i18n:host.mapUrl.label",
                TextInput::new().name("map_url").value(url),
                None,
            )
            .into(),
        );
    }
    children.push(
        field(
            config,
            ctx,
            "photo",
            "i18n:host.photo.label",
            ImageUpload::new().name("photo").value(config.photo.clone()),
            None,
        )
        .into(),
    );
    let same = ctx.input_bool("barrier_same_as_access", config.barrier_same_as_access);
    children.push(
        ToggleRow::new()
            .name("barrier_same_as_access")
            .label("i18n:host.barrierSame.label")
            .description("i18n:host.barrierSame.help")
            .checked(same)
            .into(),
    );
    children.push(
        secret_field(
            config,
            ctx,
            "parking_code",
            "i18n:host.parkingCode.label",
            &config.parking_code,
            "i18n:host.parkingCode.placeholder",
        )
        .visibleWhen("barrier_same_as_access=false")
        .into(),
    );
    children.push(reveal_field(config.reveal_policy).into());
    section("spot", children)
}

/// §2.1 La position de la borne, sur la carte : le repère ⚡ du livret. Sans épingle enregistrée,
/// l'adresse du logement (§4).
fn position_picker(config: &ModuleConfig, ctx: &HostContext) -> AddressMapPicker {
    let saved = config.charger_lat.zip(config.charger_lng);
    let address = match config.charger_address.trim() {
        "" if saved.is_none() => ctx.property.address.clone().unwrap_or_default(),
        address => address.to_string(),
    };
    let picker = AddressMapPicker::new()
        .addressName("charger_address")
        .latName("charger_lat")
        .lngName("charger_lng")
        .address(address)
        .label("i18n:host.position.label")
        .hint("i18n:host.position.hint");
    let home = ctx.property.coordinates.map(|c| (c.lat, c.lng));
    match saved.or(home) {
        Some((lat, lng)) => picker.lat(lat).lng(lng),
        None => picker,
    }
}

/// Le moment où les codes s'affichent : pas un bloc de la spec, mais les deux codes de la page
/// le suivent, et l'hôte doit pouvoir le changer.
fn reveal_field(policy: RevealPolicy) -> Field {
    let choices = [
        (RevealPolicy::Always, "always"),
        (RevealPolicy::HoursBefore24, "hoursBefore24"),
        (RevealPolicy::DayBefore16h, "dayBefore16h"),
        (RevealPolicy::AtCheckin, "atCheckin"),
    ];
    Field::new()
        .name("reveal_policy")
        .label("i18n:host.section.reveal")
        .child(
            ChoiceList::new()
                .name("reveal_policy")
                .value(policy.as_wire())
                .layout(ChoiceListLayout::List)
                .choices(
                    choices
                        .iter()
                        .map(|(policy, key)| {
                            ChoiceOption::new(policy.as_wire(), format!("i18n:host.reveal.{key}"))
                                .description(format!("i18n:host.reveal.{key}.desc"))
                        })
                        .collect(),
                ),
        )
        .child(FieldHint::new().text("i18n:host.section.reveal.help"))
}

// ── 2.2 La borne ─────────────────────────────────────────────────────────────

fn charger_section(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let chosen = config.charger_type();
    let cards: Vec<Component> = CHARGER_TYPES
        .iter()
        .map(|kind| {
            SelectableCard::new()
                .name("charger_type")
                .value(*kind)
                .label(format!("i18n:host.chargerType.label.{kind}"))
                .selected(*kind == chosen)
                .into()
        })
        .collect();

    // Un pas d'un kW depuis 0 : 0 dit « non renseignée », et un formulaire jamais touché envoie 0
    // plutôt que le minimum de la spec. Les bornes 2,3 – 350 sont celles de `problems`.
    let mut power = NumberInput::new().name("power_kw").min(0.0).max(POWER_KW.1);
    if let Some(kw) = config.power() {
        power = power.value(kw);
    }
    let estimate = config
        .power()
        .and_then(|kw| {
            let hours = charge_hours(kw).to_string();
            t!("guest.power.estimate", hours = &hours)
                .ok()
                .filter(|text| text.contains(&hours))
        })
        .unwrap_or_else(|| "i18n:host.power.hint".to_string());

    let children: Vec<Component> = vec![
        field(
            config,
            ctx,
            "charger_type",
            "i18n:host.chargerType.label",
            Grid::new().columns(2).gap(10.0).children(cards),
            None,
        )
        .required(true)
        .into(),
        field(
            config,
            ctx,
            "power_kw",
            "i18n:host.power.label",
            power,
            Some(&estimate),
        )
        .into(),
        field(
            config,
            ctx,
            "cable_provided",
            "i18n:host.cable.label",
            // `true` / `false` : le formulaire les envoie en booléens, le champ reste un interrupteur.
            RadioGroup::new()
                .name("cable_provided")
                .options(vec![
                    ChoiceOption::new("true", "i18n:host.cable.yes"),
                    ChoiceOption::new("false", "i18n:host.cable.no"),
                ])
                .value(config.cable_provided().to_string()),
            None,
        )
        .required(true)
        .into(),
        secret_field(
            config,
            ctx,
            "charger_pin",
            "i18n:host.chargerPin.label",
            &config.charger_pin,
            "i18n:host.chargerPin.placeholder",
        )
        .into(),
        field(
            config,
            ctx,
            "instructions",
            "i18n:host.instructions.label",
            TextArea::new()
                .name("instructions")
                .value(
                    config
                        .instructions
                        .as_ref()
                        .map(|text| text.host_value(ctx))
                        .unwrap_or_default(),
                )
                .placeholder("i18n:host.instructions.placeholder"),
            None,
        )
        .into(),
    ];
    section("charger", children)
}

// ── 2.3 Tarif et réservation ─────────────────────────────────────────────────

fn pricing_section(config: &ModuleConfig, ctx: &HostContext, pricing: &str) -> Component {
    let mut children: Vec<Component> = vec![field(
        config,
        ctx,
        "pricing",
        "i18n:host.pricing.label",
        ChoiceList::new()
            .name("pricing")
            .value(pricing)
            .emitOnChange(true)
            .layout(ChoiceListLayout::List)
            .choices(
                PRICINGS
                    .iter()
                    .map(|p| ChoiceOption::new(*p, format!("i18n:host.pricing.label.{p}")))
                    .collect(),
            ),
        None,
    )
    .required(true)
    .into()];
    // Masqué, pas grisé, tant que la recharge est incluse (règles communes §2.2).
    if pricing != "included" {
        children.push(
            field(
                config,
                ctx,
                "price",
                "i18n:host.price.label",
                TextInput::new()
                    .name("price")
                    .value(config.price.clone())
                    .placeholder("i18n:host.price.placeholder"),
                None,
            )
            .required(true)
            .into(),
        );
    }
    let booking = ctx.input_bool("booking_required", config.booking_required);
    children.push(
        ToggleRow::new()
            .name("booking_required")
            .label("i18n:host.booking.label")
            .description("i18n:host.booking.hint")
            .checked(booking)
            .into(),
    );
    children.push(
        field(
            config,
            ctx,
            "booking_note",
            "i18n:host.bookingNote.label",
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
            None,
        )
        .visibleWhen("booking_required=true")
        .into(),
    );
    section("pricing", children)
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// One card of the settings page (§2.2 of the common rules): a `Section`, never a `Card`.
fn section(key: &str, children: Vec<Component>) -> Component {
    Section::new()
        .title(format!("i18n:host.section.{key}"))
        .children(children)
        .into()
}

/// Libellé, contrôle, l'aide dessous, et le message de [`ModuleConfig::error_of`] s'il y en a un.
fn field(
    config: &ModuleConfig,
    ctx: &HostContext,
    name: &str,
    label: &str,
    control: impl Into<Component>,
    hint: Option<&str>,
) -> Field {
    let mut field = Field::new().name(name).label(label).child(control);
    if let Some(hint) = hint {
        field = field.child(FieldHint::new().text(hint));
    }
    match config.error_of(name) {
        Some(error) => field.error(error.get(&ctx.locale).to_string()),
        None => field,
    }
}

/// A code is never sent back to the form: blank keeps it (the platform ignores `""`). The
/// placeholder says so only when there is one to keep.
fn secret_field(
    config: &ModuleConfig,
    ctx: &HostContext,
    name: &str,
    label: &str,
    saved: &str,
    keep: &str,
) -> Field {
    let mut input = SecretInput::new().name(name).value(String::new());
    if !saved.trim().is_empty() {
        input = input.placeholder(keep);
    }
    field(config, ctx, name, label, input, None)
}
