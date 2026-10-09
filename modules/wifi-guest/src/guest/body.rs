//! Shared guest SDUI body for guest Wi-Fi.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::SecretState;
use portaki_sdk::sdui::primitives::{Eyebrow, InfoBanner, KeyValue, QRCode, Text};

use super::load::{password_display, GuestData};
use super::qr::wifi_payload;
use crate::config::Network;

fn kv_row(key_i18n: &str, value: &str, mono: bool) -> Component {
    let mut row = KeyValue::new().key(key_i18n).value(value);
    if mono {
        row = row.mono(true);
    }
    Component::KeyValue(row)
}

fn push_reveal_banner(children: &mut Vec<Component>, data: &GuestData) {
    let any_secret = data
        .networks
        .iter()
        .any(|n| !password_display(data, n).is_empty());
    if data.password_revealed || !any_secret {
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
/// An open network has no secret to hold back, so its code shows at any time. The host may also
/// turn the codes off (`show_qr`).
fn push_qr_code(children: &mut Vec<Component>, data: &GuestData, network: &Network) {
    if !data.config.show_qr {
        return;
    }
    let password = password_display(data, network);
    if !password.is_empty() && !data.password_revealed {
        return;
    }
    let Some(payload) = wifi_payload(
        &network.ssid,
        &network.password,
        network.security,
        network.hidden,
    ) else {
        return;
    };
    children.push(Component::QRCode(QRCode::new().value(payload)));
}

/// Où ce corps est dessiné — la carte d'accueil, ou la feuille qu'elle ouvre.
///
/// La carte montre le premier réseau et a le message de l'hôte en sous-titre ; la feuille montre
/// tous les réseaux, le message en tête (un portail captif se dit avant tout), et le rappel de
/// sécurité. Un seul paramètre le dit, plutôt que deux booléens qu'on finit par passer à l'envers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    Card,
    Sheet,
}

pub fn build_wifi_body(data: &GuestData, placement: Placement) -> Vec<Component> {
    let mut children = Vec::new();

    if placement == Placement::Sheet {
        if let Some(note) = data.config.note_text(&data.locale) {
            children.push(Text::new().text(note).variant(TextVariant::Body).into());
        }
        children.push(Component::InfoBanner(
            InfoBanner::new()
                .title("i18n:guest.security.title")
                .message("i18n:guest.security.message"),
        ));
    }

    push_reveal_banner(&mut children, data);

    let shown: &[Network] = match placement {
        Placement::Card => &data.networks[..data.networks.len().min(1)],
        Placement::Sheet => &data.networks,
    };
    let several = shown.len() > 1;
    for network in shown {
        if several {
            let label = network
                .label
                .as_ref()
                .map(|label| label.get(&data.locale).trim().to_string())
                .filter(|label| !label.is_empty())
                .unwrap_or_else(|| network.ssid.trim().to_string());
            children.push(Eyebrow::new().text(label).into());
        }
        // En mono comme le mot de passe : c'est ce qu'on recopie à la main quand le code échoue,
        // et une proportionnelle y confond l avec I, 0 avec O.
        children.push(kv_row("i18n:guest.ssid", network.ssid.trim(), true));

        let password = password_display(data, network);
        if !password.is_empty() {
            // La révélation et la copie tiennent dans la ligne, comme la maquette les dessine.
            let mut row = KeyValue::new()
                .key("i18n:guest.password")
                .value(password)
                .mono(true);
            if data.password_revealed {
                row = row.copy(true).copyLabel("i18n:guest.copyPassword");
            } else {
                row = row.secret(SecretState::hidden(data.reveal_at_label.clone()));
            }
            children.push(Component::KeyValue(row));
        }

        push_qr_code(&mut children, data, network);

        if network.hidden {
            children.push(
                Text::new()
                    .text("i18n:guest.hidden.help")
                    .variant(TextVariant::Caption)
                    .into(),
            );
        }
    }

    children
}
