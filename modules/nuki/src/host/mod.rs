//! Host dashboard surfaces.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Button, Card, Field, Form, Page, SecretInput, Text, TextInput};
use portaki_sdk::sdui::surface::Surface;

use crate::config::load_config;

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(_ctx: HostContext) -> Surface {
    let config = load_config().unwrap_or_default();

    let save_action = crate::ids::module_id().command(
        crate::ids::UPDATE_CONFIG,
        crate::commands::UpdateConfigArgs {
            smartlock_id: config.smartlock_id.clone(),
            keypad_code: config.keypad_code.clone(),
            device_name: config.device_name.clone(),
        },
    );

    let form_children: Vec<Component> = vec![
        Card::new()
            .title("i18n:host.section.device")
            .subtitle("i18n:host.section.device.help")
            .icon("lock")
            .children(vec![
                Field::new()
                    .name("smartlock_id")
                    .label("i18n:host.smartlockId.label")
                    .child(
                        TextInput::new()
                            .name("smartlock_id")
                            .value(config.smartlock_id)
                            .placeholder("i18n:host.smartlockId.placeholder"),
                    )
                    .into(),
                Field::new()
                    .name("device_name")
                    .label("i18n:host.deviceName.label")
                    .child(
                        TextInput::new()
                            .name("device_name")
                            .value(config.device_name)
                            .placeholder("i18n:host.deviceName.placeholder"),
                    )
                    .into(),
            ])
            .into(),
        Card::new()
            .title("i18n:host.section.keypad")
            .subtitle("i18n:host.section.keypad.help")
            .icon("key")
            .children(vec![Field::new()
                .name("keypad_code")
                .label("i18n:host.keypadCode.label")
                .child(
                    SecretInput::new()
                        .name("keypad_code")
                        .value(config.keypad_code)
                        .placeholder("i18n:host.keypadCode.placeholder"),
                )
                .into()])
            .into(),
        Text::new()
            .text("i18n:host.main.help")
            .variant(TextVariant::Caption)
            .into(),
        Button::new()
            .label("i18n:host.save")
            .action(save_action)
            .into(),
    ];

    Surface::new(
        Page::new()
            .title("i18n:surface.host.main.title")
            .child(
                Text::new()
                    .text("i18n:surface.host.main.subtitle")
                    .variant(TextVariant::Body),
            )
            .child(Form::new().children(form_children)),
    )
    .with_id(crate::ids::HOST_MAIN)
}
