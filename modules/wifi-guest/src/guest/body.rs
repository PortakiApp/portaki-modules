//! Shared guest SDUI body for guest Wi-Fi.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{Button, InfoBanner, KeyValue, Text};

use super::load::{password_display, GuestData};

fn kv_row(key_i18n: &str, value: &str, mono: bool) -> Component {
    let mut row = KeyValue::new().key(key_i18n).value(value);
    if mono {
        row = row.mono(true);
    }
    Component::KeyValue(row)
}

fn push_reveal_banner(children: &mut Vec<Component>, data: &GuestData) {
    if data.password_revealed || data.config.password.trim().is_empty() {
        return;
    }
    let Some(message) = data.reveal_locked_message.as_ref() else {
        return;
    };
    children.push(Component::InfoBanner(
        InfoBanner::new()
            .title("i18n:guest.reveal.lockedTitle")
            .message(message.clone()),
    ));
}

pub fn build_wifi_body(data: &GuestData, show_security_banner: bool) -> Vec<Component> {
    let mut children = Vec::new();

    if show_security_banner {
        children.push(Component::InfoBanner(
            InfoBanner::new()
                .title("i18n:guest.security.title")
                .message("i18n:guest.security.message"),
        ));
    }

    push_reveal_banner(&mut children, data);

    let ssid = data.config.ssid.trim();
    if !ssid.is_empty() {
        children.push(kv_row("i18n:guest.ssid", ssid, false));
    }

    let password = data.config.password.trim();
    if !password.is_empty() {
        children.push(kv_row("i18n:guest.password", &password_display(data), true));
        if data.password_revealed {
            children.push(Component::Button(
                Button::new()
                    .label("i18n:guest.copyPassword")
                    .variant(ButtonVariant::Outline)
                    .action(Action::copy(
                        password.to_string(),
                        Some("i18n:guest.copy.toast".into()),
                    )),
            ));
        }
    }

    if let Some(hint) = data.config.hint_text() {
        children.push(Text::new().text(hint).variant(TextVariant::Caption).into());
    }

    if let Some(steps) = data.config.connection_steps_text() {
        children.push(Text::new().text(steps).variant(TextVariant::Body).into());
    }

    children
}
