//! Host dashboard surface — design `editorNuki` / `nuki-editor-v1`.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{
    Card, Field, Form, InfoBanner, Page, SecretInput, Stack, TextInput,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::load_config;

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(_ctx: HostContext) -> Surface {
    let config = load_config().unwrap_or_default();

    let form_children: Vec<Component> = vec![
        InfoBanner::new()
            .title("i18n:host.banner.title")
            .message("i18n:host.banner.message")
            .into(),
        Card::new()
            .title("i18n:host.section.device")
            .subtitle("i18n:host.section.device.help")
            .icon("lock")
            .children(vec![
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
                    .name("keypad_code")
                    .label("i18n:host.keypadCode.label")
                    .child(
                        SecretInput::new()
                            .name("keypad_code")
                            .value(config.keypad_code)
                            .placeholder("i18n:host.keypadCode.placeholder"),
                    )
                    .into(),
            ])
            .into(),
    ];

    // No Page title / Save — the modules sheet owns chrome + footer Save.
    Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(16.0).children(form_children))),
    )
    .with_id(crate::ids::HOST_MAIN)
}
