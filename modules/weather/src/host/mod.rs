//! Host dashboard surface — the weather settings drawer (spec Météo §2).
//!
//! Two cards, in the spec's order: where to forecast, and what to show. The drawer chrome (title,
//! switch, Publier) belongs to the dashboard.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    AddressMapPicker, Card, Field, Form, InlineNotice, Page, Text, TextInput, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::ModuleConfig;

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
        .child(display_card(&config));
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
        (Some(address), Some(_)) => Field::new()
            .label("i18n:host.location.label")
            .child(Text::new().text(address))
            .into(),
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
            .description("i18n:host.location.help")
            .checked(config.location_override)
            .into(),
    );
    if config.location_override {
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
            .child(picker);
        if let Some(error) = config.pin_error() {
            field = field.error(error.get(&ctx.locale).to_string());
        }
        children.push(field.into());
        if config.override_is_far(ctx.property.coordinates) {
            children.push(
                InlineNotice::new()
                    .tone(Tone::Warning)
                    .message("i18n:host.locationOverride.far")
                    .into(),
            );
        }
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

/// §2.2 Affichage.
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
        .into()
}
