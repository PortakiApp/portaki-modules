//! Host dashboard surface — flat SDUI form for the modules drawer.
//!
//! Matches Portaki Dashboard.dc.html `sduiForm` for `wifi-guest`
//! (`configMode: "drawer"`): warning alert + labeled fields, no nested Cards.
//! Drawer chrome (title, enable toggle, Annuler / Enregistrer) stays in the host.

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    ChoiceList, Field, FieldHint, Form, InfoBanner, Page, SecretInput, Stack, TextArea, TextInput,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{ModuleConfig, RevealPolicy, WifiSecurity};

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyModuleSheet,
    label_key = "catalog.host.main",
    icon = IconName::Wifi
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;

    let form_children: Vec<Component> = vec![
        InfoBanner::new()
            .tone(Tone::Warning)
            .message("i18n:host.main.warning")
            .into(),
        Field::new()
            .name("ssid")
            .label("i18n:host.ssid.label")
            .required(true)
            .children(vec![
                FieldHint::new().text("i18n:host.ssid.desc").into(),
                TextInput::new()
                    .name("ssid")
                    .value(config.ssid.clone())
                    .placeholder("i18n:host.ssid.placeholder")
                    .into(),
            ])
            .into(),
        Field::new()
            .name("password")
            .label("i18n:host.password.label")
            .children(vec![
                FieldHint::new().text("i18n:host.password.desc").into(),
                SecretInput::new()
                    .name("password")
                    .value(String::new())
                    .placeholder("i18n:host.password.placeholder")
                    .into(),
            ])
            .into(),
        Field::new()
            .name("hint")
            .label("i18n:host.hint.label")
            .children(vec![
                FieldHint::new().text("i18n:host.hint.desc").into(),
                TextInput::new()
                    .name("hint")
                    .value(host_value(config.hint.as_ref(), &ctx))
                    .placeholder("i18n:host.hint.placeholder")
                    .into(),
            ])
            .into(),
        Field::new()
            .name("connection_steps")
            .label("i18n:host.connectionSteps.label")
            .children(vec![
                FieldHint::new()
                    .text("i18n:host.connectionSteps.desc")
                    .into(),
                TextArea::new()
                    .name("connection_steps")
                    .value(host_value(config.connection_steps.as_ref(), &ctx))
                    .placeholder("i18n:host.connectionSteps.placeholder")
                    .into(),
            ])
            .into(),
        Field::new()
            .name("security")
            .label("i18n:config.security")
            .children(vec![
                FieldHint::new().text("i18n:config.security.desc").into(),
                security_choice_list(config.security).into(),
            ])
            .into(),
        Field::new()
            .name("reveal_policy")
            .label("i18n:host.section.reveal")
            .children(vec![
                FieldHint::new()
                    .text("i18n:host.section.reveal.help")
                    .into(),
                reveal_choice_list(config.reveal_policy).into(),
            ])
            .into(),
    ];

    // No Page title / Save — the modules drawer owns chrome + footer Save.
    Ok(Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(20.0).children(form_children))),
    )
    .with_id(MAIN))
}

/// Le chiffrement du réseau, tel que le code QR doit l'annoncer.
///
/// Trois choix et non quatre : WPA couvre WPA2 et WPA3, que les téléphones lisent de la même façon.
/// « Réseau ouvert » n'est pas un défaut mais un cas réel — un portail captif n'a pas de mot de
/// passe, et un code QR qui en annonce un ferait demander au téléphone une clé qui n'existe pas.
fn security_choice_list(security: WifiSecurity) -> ChoiceList {
    ChoiceList::new()
        .name("security")
        .value(security.as_wire())
        .choices(vec![
            ChoiceOption::new(WifiSecurity::Wpa.as_wire(), "i18n:config.security.wpa")
                .icon(IconName::Lock),
            ChoiceOption::new(WifiSecurity::Wep.as_wire(), "i18n:config.security.wep")
                .icon(IconName::Lock),
            ChoiceOption::new(
                WifiSecurity::Nopass.as_wire(),
                "i18n:config.security.nopass",
            )
            .icon(IconName::Wifi),
        ])
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

fn host_value(text: Option<&I18nText>, ctx: &HostContext) -> String {
    text.map(|text| text.host_value(ctx).to_string())
        .unwrap_or_default()
}
