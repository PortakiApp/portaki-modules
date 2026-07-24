//! Host dashboard surface — design `editorEvParking` / `evparking-editor-v1`.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{
    Card, ChoiceList, Field, Form, Page, SecretInput, Stack, TextArea, TextInput,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{load_config, RevealPolicy};

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(_ctx: HostContext) -> Surface {
    let config = load_config().unwrap_or_default();

    let form_children: Vec<Component> = vec![
        Card::new()
            .title("i18n:host.section.spot")
            .subtitle("i18n:host.section.spot.help")
            .icon("zap")
            .children(vec![
                Field::new()
                    .name("spot_label")
                    .label("i18n:host.spotLabel.label")
                    .child(
                        TextInput::new()
                            .name("spot_label")
                            .value(config.spot_label.clone())
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
            .icon("info-circle")
            .children(vec![Field::new()
                .name("instructions")
                .label("i18n:host.instructions.label")
                .child(
                    // TipTap preferred in design; TextArea until guest renders rich HTML.
                    TextArea::new()
                        .name("instructions")
                        .value(config.instructions.clone().unwrap_or_default())
                        .placeholder("i18n:host.instructions.placeholder"),
                )
                .into()])
            .into(),
        Card::new()
            .title("i18n:host.section.reveal")
            .subtitle("i18n:host.section.reveal.help")
            .icon("clock-circle")
            .children(vec![reveal_choice_list(config.reveal_policy).into()])
            .into(),
    ];

    // No Page title / Save — the modules sheet owns chrome + footer Save.
    Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(16.0).children(form_children))),
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
