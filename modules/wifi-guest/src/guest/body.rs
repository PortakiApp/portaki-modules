//! Shared guest SDUI body for guest Wi-Fi.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::SecretState;
use portaki_sdk::sdui::primitives::{InfoBanner, KeyValue, QRCode, Text};

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
    let Some(payload) = wifi_payload(
        &data.config.ssid,
        password,
        data.config.security,
        data.config.hidden,
    ) else {
        return;
    };
    children.push(Component::QRCode(QRCode::new().value(payload)));
}

/// Où ce corps est dessiné — la carte d'accueil, ou la feuille qu'elle ouvre.
///
/// Les deux ne portent pas la même chose : la carte a un sous-titre, la feuille non ; la feuille a
/// la place du rappel de sécurité, la carte non. Un seul paramètre le dit, plutôt que deux
/// booléens qu'on finit par passer à l'envers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Carte d'accueil : la note de l'hôte est déjà son sous-titre.
    Card,
    /// Feuille de détail : rien au-dessus, tout se dit ici.
    Sheet,
}

pub fn build_wifi_body(data: &GuestData, placement: Placement) -> Vec<Component> {
    let mut children = Vec::new();

    if placement == Placement::Sheet {
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
        // La révélation et la copie tiennent dans la ligne, comme la maquette les dessine (§2.2).
        // Un bouton « Copier le mot de passe » dessous répétait une action que la ligne porte
        // déjà, et le posait sous le QR qu'il faut regarder.
        let mut row = KeyValue::new()
            .key("i18n:guest.password")
            .value(password_display(data))
            .mono(true);
        if data.password_revealed {
            row = row.copy(true).copyLabel("i18n:guest.copyPassword");
        } else {
            row = row.secret(SecretState::hidden(data.reveal_at_label.clone()));
        }
        children.push(Component::KeyValue(row));
    }

    push_qr_code(&mut children, data);

    // La note de l'hôte est le sous-titre de la carte : la redire dans le corps l'affichait deux
    // fois sur le même écran. Dans la feuille, où il n'y a pas de sous-titre, elle reste.
    if placement == Placement::Sheet {
        if let Some(hint) = data.config.hint_text(&data.locale) {
            children.push(Text::new().text(hint).variant(TextVariant::Caption).into());
        }
    }

    if let Some(steps) = data.config.connection_steps_text(&data.locale) {
        children.push(Text::new().text(steps).variant(TextVariant::Body).into());
    }

    children
}
