//! Shared guest SDUI body for EV parking.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::KeyValueLayout;
use portaki_sdk::sdui::primitives::{Grid, InfoBanner, KeyValue, Link, Text};

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

/// Une tuile de code (§2.3), ou rien quand l'hôte n'a pas rempli ce code-là.
///
/// La tuile porte sa copie au lieu d'un bouton en dessous : « valeur + Copier » est une seule
/// chose pour le voyageur, et un bouton séparé s'éloignait de la valeur dès qu'il y en avait deux.
///
/// Le nombre de tuiles n'est pas notre affaire : une seule occupe la largeur parce que la grille du
/// livret le décide (§0.2), pas parce que le module l'aurait demandé.
fn push_secret_tile(
    children: &mut Vec<Component>,
    data: &GuestData,
    key_i18n: &str,
    copy_label: &str,
    icon: IconName,
    code: &str,
) {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return;
    }
    let mut tile = KeyValue::new()
        .key(key_i18n)
        .value(secret_display(data, trimmed))
        .layout(KeyValueLayout::Tile)
        .icon(icon)
        .mono(true);
    // Pas de copie sur une valeur masquée : il n'y aurait que des points à mettre dans le presse-
    // papiers, et le voyageur croirait tenir son code.
    if data.secrets_revealed {
        // `copyLabel` et non `copy_label` : le SDK nomme ses accesseurs d'après le champ du
        // contrat, qui est en camelCase.
        #[allow(non_snake_case)]
        {
            tile = tile.copy(true).copyLabel(copy_label);
        }
    }
    children.push(Component::KeyValue(tile));
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

    // Les deux codes côte à côte, en grille : ce sont les deux choses qu'on cherche en arrivant
    // au parking, et une tuile sous l'autre fait descendre la seconde sous le pli.
    let mut tiles = Vec::new();
    push_secret_tile(
        &mut tiles,
        data,
        "i18n:guest.parkingCode",
        "i18n:guest.copyParkingCode",
        IconName::Lock,
        &data.config.parking_code,
    );
    push_secret_tile(
        &mut tiles,
        data,
        "i18n:guest.chargerPin",
        "i18n:guest.copyChargerPin",
        IconName::Zap,
        &data.config.charger_pin,
    );
    if !tiles.is_empty() {
        children.push(Component::Grid(
            Grid::new()
                .minColumnWidth(130.0)
                .plain(true)
                .children(tiles),
        ));
    }

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
