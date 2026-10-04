//! Guest home booklet card.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::{Button, Card, Text};
use portaki_sdk::sdui::surface::Surface;

use super::body::build_spots_body;
use super::load::GuestData;

pub fn build_home_card(data: &GuestData) -> Surface {
    let open_detail = Action::open_overlay(
        OverlayPresentation::BottomSheet,
        crate::guest::EXPLORE_DETAIL,
        OverlayArgs::new()
            .icon(IconName::MapPin)
            .title("i18n:home.card.title"),
    );

    let mut children = build_spots_body(data, false);

    // « Voir tout sur la carte » : le plan est dans la sous-page, pas sur la carte d'accueil —
    // une carte interactive au milieu du livret attrape le défilement du doigt (§2.12).
    if !data.spots.is_empty() {
        children.push(
            Button::new()
                .label("i18n:guest.seeOnMap")
                .variant(ButtonVariant::Outline)
                .action(open_detail.clone())
                .into(),
        );
    }

    // La mention ne vaut que pour les adresses de l'hôte : la coller sous une section partenaire
    // dirait « sans partenariat » juste au-dessus d'un lien commissionné.
    let partnered = data.activities.is_some() || data.tiqets.is_some() || data.viator.is_some();
    if !data.spots.is_empty() && !partnered {
        children.push(
            Text::new()
                .text("i18n:guest.hostPicks")
                .variant(TextVariant::Caption)
                .into(),
        );
    }

    let mut card = Card::new()
        .icon(IconName::MapPin)
        .title("i18n:home.card.title")
        .action(open_detail);
    if !data.host_name.is_empty() {
        card = card.subtitle(
            t!("home.card.subtitle", host = data.host_name.clone())
                .unwrap_or_else(|_| "i18n:guest.teaser".into()),
        );
    }

    Surface::new(card.children(children)).with_id(crate::guest::HOME_CARD)
}
