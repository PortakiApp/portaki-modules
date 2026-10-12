//! Host dashboard surface — the weather settings drawer (spec Météo §2).
//!
//! Two cards, in the spec's order: where to forecast, and what to show. The drawer chrome (title,
//! switch, Publier) belongs to the dashboard.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    AddressMapPicker, Card, Field, FieldHint, Form, InlineNotice, Page, Pill, TextInput, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::ModuleConfig;
use crate::weather::city_from_address;

/// Le repli ouvert dans le brouillon.
const OVERRIDE_ON: &str = "location_override=true";

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyModuleSheet,
    label_key = "catalog.host.main",
    icon = IconName::CloudSun
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    let form = Form::new()
        .child(location_card(&config, &ctx))
        .child(display_card(&config))
        .child(public_card(&config));
    Ok(Surface::new(Page::new().child(form)).with_id(MAIN))
}

/// §2.1 Lieu : l'adresse du logement, en lecture seule ; une autre position seulement en repli.
fn location_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let mut children: Vec<Component> = Vec::new();
    let address = ctx
        .property
        .address
        .as_deref()
        .map(str::trim)
        .filter(|a| !a.is_empty());
    children.push(match (address, ctx.property.coordinates) {
        // Lecture seule : sans `name`, le formulaire ne l'envoie pas.
        (Some(address), Some(_)) => {
            let place = match city_from_address(address) {
                Some(commune) => t!("host.location.fromAddress", commune = commune),
                None => t!("host.location.fromAddressOnly"),
            };
            Field::new()
                .label("i18n:host.location.label")
                .child(
                    InlineNotice::new()
                        .tone(Tone::Neutral)
                        .message(place.unwrap_or_default()),
                )
                .child(FieldHint::new().text("i18n:host.location.help"))
                .into()
        }
        // Cas 4 : sans adresse géocodée, pas de prévisions — sauf position corrigée.
        _ => InlineNotice::new()
            .tone(Tone::Warning)
            .message("i18n:host.location.missing")
            .into(),
    });
    children.push(
        ToggleRow::new()
            .name("location_override")
            .label("i18n:host.locationOverride.label")
            .checked(config.location_override)
            .into(),
    );
    // Le sélecteur suit l'interrupteur du brouillon (masqué, pas grisé) ; masqué, il n'est pas
    // envoyé et la position enregistrée reste.
    let mut picker = AddressMapPicker::new()
        .addressName("location_address")
        .latName("location_lat")
        .lngName("location_lng")
        .address(config.location_address.clone());
    // La position enregistrée, ou aucune — jamais 0, 0 : chaque enregistrement l'effacerait.
    if let (Some(lat), Some(lng)) = (config.location_lat, config.location_lng) {
        picker = picker.lat(lat).lng(lng);
    }
    let mut field = Field::new()
        .name("location_lat")
        .label("i18n:host.locationOverride.position")
        .visibleWhen(OVERRIDE_ON)
        .child(picker);
    if let Some(error) = config.pin_error() {
        field = field.error(error.get(&ctx.locale).to_string());
    }
    children.push(field.into());
    if config.override_is_far(ctx.property.coordinates) {
        children.push(
            Field::new()
                .visibleWhen(OVERRIDE_ON)
                .child(
                    InlineNotice::new()
                        .tone(Tone::Warning)
                        .message("i18n:host.locationOverride.far"),
                )
                .into(),
        );
    }
    let mut label = Field::new()
        .name("location_label")
        .label("i18n:host.locationLabel.label")
        .required(false)
        .child(
            TextInput::new()
                .name("location_label")
                .value(config.location_label.clone())
                .placeholder("i18n:host.locationLabel.placeholder"),
        );
    if let Some(error) = config.label_error() {
        label = label.error(error.get(&ctx.locale).to_string());
    }
    children.push(label.into());

    Card::new()
        .title("i18n:host.section.location")
        .children(children)
        .into()
}

/// §2.2 Affichage. La vigilance est annoncée, pas proposée : aucune source officielle encore.
fn display_card(config: &ModuleConfig) -> Component {
    Card::new()
        .title("i18n:host.section.display")
        .child(
            ToggleRow::new()
                .name("show_upcoming")
                .label("i18n:host.showUpcoming.label")
                .description("i18n:host.showUpcoming.help")
                .checked(config.show_upcoming),
        )
        .child(
            Field::new()
                .label("i18n:host.alerts.label")
                .child(Pill::new().label("i18n:host.alerts.soon"))
                .child(FieldHint::new().text("i18n:host.alerts.help")),
        )
        .child(FieldHint::new().text("i18n:host.display.hint"))
        .into()
}

/// Carte « Page publique » (règles communes de la tranche 5) : le bloc Climat de la page publique.
fn public_card(config: &ModuleConfig) -> Component {
    Card::new()
        .title("i18n:host.section.public")
        .child(
            ToggleRow::new()
                .name("public_enabled")
                .label("i18n:host.publicEnabled.label")
                .checked(config.public_enabled),
        )
        .child(
            ToggleRow::new()
                .name("public_monthly_averages")
                .label("i18n:host.publicMonthlyAverages.label")
                .description("i18n:host.publicMonthlyAverages.help")
                .checked(config.public_monthly_averages),
        )
        .into()
}
