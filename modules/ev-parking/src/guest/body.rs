//! Shared guest SDUI body for EV parking.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{Button, InfoBanner, KeyValue, Link, Text};

use super::load::{has_any_secret, secret_display, GuestData};

fn external_action(url: &str) -> Action {
    Action::external(url)
}

fn kv_row(key_i18n: &str, value: &str, mono: bool) -> Component {
    let mut row = KeyValue::new().key(key_i18n).value(value);
    if mono {
        row = row.mono(true);
    }
    Component::KeyValue(row)
}

fn push_reveal_banner(children: &mut Vec<Component>, data: &GuestData) {
    if data.secrets_revealed || !has_any_secret(&data.config) {
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

fn push_secret_row(
    children: &mut Vec<Component>,
    data: &GuestData,
    key_i18n: &str,
    copy_label: &str,
    copy_toast: &str,
    code: &str,
) {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return;
    }
    children.push(kv_row(key_i18n, &secret_display(data, trimmed), true));
    if data.secrets_revealed {
        children.push(Component::Button(
            Button::new()
                .label(copy_label)
                .variant(ButtonVariant::Outline)
                .action(Action::copy(trimmed.to_string(), Some(copy_toast.into()))),
        ));
    }
}

pub fn build_ev_parking_body(data: &GuestData) -> Vec<Component> {
    let mut children = Vec::new();

    push_reveal_banner(&mut children, data);

    if let Some(spot) = data.config.spot_text(&data.locale) {
        children.push(kv_row("i18n:guest.spot", spot, false));
    }

    if let Some(url) = data.config.map_url_text() {
        children.push(Component::Link(
            Link::new()
                .label("i18n:guest.openMap")
                .href(url.to_string())
                .action(external_action(url)),
        ));
    }

    push_secret_row(
        &mut children,
        data,
        "i18n:guest.parkingCode",
        "i18n:guest.copyParkingCode",
        "i18n:guest.copy.parkingCode.toast",
        &data.config.parking_code,
    );

    push_secret_row(
        &mut children,
        data,
        "i18n:guest.chargerPin",
        "i18n:guest.copyChargerPin",
        "i18n:guest.copy.chargerPin.toast",
        &data.config.charger_pin,
    );

    if let Some(instructions) = data.config.instructions_text(&data.locale) {
        children.push(
            Text::new()
                .text(instructions)
                .variant(TextVariant::Caption)
                .into(),
        );
    }

    children
}
