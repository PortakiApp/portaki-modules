//! Host dashboard surface — flat SDUI form for the modules drawer.
//!
//! Matches Portaki Dashboard.dc.html `sduiForm` for `wifi-guest`
//! (`configMode: "drawer"`): warning alert + labeled fields, no nested Cards.
//! Drawer chrome (title, enable toggle, Annuler / Enregistrer) stays in the host.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    ChoiceList, Field, FieldHint, Form, InfoBanner, Page, SecretInput, Stack, TextArea, TextInput,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{load_config, RevealPolicy};

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(_ctx: HostContext) -> Surface {
    let config = load_config().unwrap_or_default();

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
            .required(true)
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
                    .value(config.hint.clone().unwrap_or_default())
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
                    .value(config.connection_steps.clone().unwrap_or_default())
                    .placeholder("i18n:host.connectionSteps.placeholder")
                    .into(),
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
    Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(20.0).children(form_children))),
    )
    .with_id(crate::ids::HOST_MAIN)
}

fn reveal_choice_list(policy: RevealPolicy) -> ChoiceList {
    ChoiceList::new()
        .name("reveal_policy")
        .value(policy.as_wire())
        .choices(vec![
            ChoiceOption::new(RevealPolicy::Always.as_wire(), "i18n:host.reveal.always")
                .description("i18n:host.reveal.always.desc")
                .icon("clock-circle"),
            ChoiceOption::new(
                RevealPolicy::HoursBefore24.as_wire(),
                "i18n:host.reveal.hoursBefore24",
            )
            .description("i18n:host.reveal.hoursBefore24.desc")
            .icon("clock-circle"),
            ChoiceOption::new(
                RevealPolicy::DayBefore16h.as_wire(),
                "i18n:host.reveal.dayBefore16h",
            )
            .description("i18n:host.reveal.dayBefore16h.desc")
            .icon("clock-circle"),
            ChoiceOption::new(
                RevealPolicy::AtCheckin.as_wire(),
                "i18n:host.reveal.atCheckin",
            )
            .description("i18n:host.reveal.atCheckin.desc")
            .icon("key"),
        ])
}
