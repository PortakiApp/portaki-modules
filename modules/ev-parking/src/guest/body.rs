//! Shared guest SDUI body for EV parking.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::{KeyValueLayout, SecretState};
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
    // Masquée, la tuile dit qu'elle s'ouvrira et quand : des points sans promesse se lisent comme
    // un code que l'hôte aurait oublié de mettre (§2.3).
    if !data.secrets_revealed {
        tile = tile.secret(SecretState::hidden(data.reveal_at_label.clone()));
    }
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

/// `enriched` : la sous-page, qui n'a pas le sous-titre de la carte pour porter l'emplacement.
pub fn build_ev_parking_body(data: &GuestData, enriched: bool) -> Vec<Component> {
    let mut children = Vec::new();

    push_reveal_banner(&mut children, data);

    // Sur la carte d'accueil, l'emplacement est déjà le sous-titre : le répéter en ligne poussait
    // les deux codes sous le pli, alors que ce sont eux qu'on vient chercher (§2.3).
    if enriched {
        if let Some(spot) = data.config.spot_text(&data.locale) {
            children.push(kv_row("i18n:guest.spot", spot, false));
        }
    }

    // Les deux codes côte à côte, en grille : ce sont les deux choses qu'on cherche en arrivant
    // au parking, et une tuile sous l'autre fait descendre la seconde sous le pli.
    let mut tiles = Vec::new();
    push_secret_tile(
        &mut tiles,
        data,
        "i18n:guest.parkingCode",
        "i18n:guest.copyParkingCode",
        // Une barrière est une affaire de voiture, pas de cadenas : c'est l'icône de la maquette,
        // et elle se distingue de l'éclair de la borne à côté.
        IconName::Car,
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

    // Le plan après les codes : on ouvre une carte quand on cherche la place, pas le code.
    if let Some(url) = data.config.map_url_text() {
        children.push(Component::Link(
            Link::new()
                .label("i18n:guest.openMap")
                .href(url.to_string())
                .action(external_action(url)),
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
