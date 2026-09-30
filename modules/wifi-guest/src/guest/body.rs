//! Shared guest SDUI body for guest Wi-Fi.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{Button, InfoBanner, KeyValue, QRCode, Text};

use super::load::{password_display, GuestData};
use super::qr::wifi_payload;

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

/// The code that joins the network without anyone typing anything (§2.2).
///
/// Nothing is drawn while the password is still held back. A code carries the key in clear inside
/// its pixels, so showing one before the reveal date would hand over exactly what the masked row
/// above it refuses — and a phone would join the network, which no mask could then undo.
///
/// An open network has no secret to hold back, so its code shows at any time.
fn push_qr_code(children: &mut Vec<Component>, data: &GuestData) {
    let password = data.config.password.trim();
    if !password.is_empty() && !data.password_revealed {
        return;
    }
    let Some(payload) = wifi_payload(&data.config.ssid, password, data.config.security) else {
        return;
    };
    children.push(Component::QRCode(QRCode::new().value(payload)));
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
        // En mono comme le mot de passe : c'est ce qu'on recopie à la main quand le code échoue,
        // et une proportionnelle y confond l avec I, 0 avec O.
        children.push(kv_row("i18n:guest.ssid", ssid, true));
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

    push_qr_code(&mut children, data);

    if let Some(hint) = data.config.hint_text(&data.locale) {
        children.push(Text::new().text(hint).variant(TextVariant::Caption).into());
    }

    if let Some(steps) = data.config.connection_steps_text(&data.locale) {
        children.push(Text::new().text(steps).variant(TextVariant::Body).into());
    }

    children
}
