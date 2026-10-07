//! Guest home booklet card.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::{Button, Card};
use portaki_sdk::sdui::surface::Surface;

use super::body::{bin_rows, build_collection_banner, compost_row, dropoff_row};
use super::load::GuestData;

/// Le nombre de bacs que la carte montre avant de renvoyer à la feuille (§2.7).
const CARD_BIN_LIMIT: usize = 3;

pub fn build_home_card(data: &GuestData) -> Surface {
    let open = || {
        Action::open_overlay(
            OverlayPresentation::BottomSheet,
            crate::guest::EXPLORE_DETAIL,
            OverlayArgs::new()
                .icon(IconName::Recycle)
                .title("i18n:home.card.title"),
        )
    };

    let mut children = build_collection_banner(data);
    let bins = bin_rows(data);
    // Trois bacs sur la carte, le reste dans la feuille (§1.9, §2.7). Un logement qui trie le
    // verre, le papier, les emballages, les biodéchets et le tout-venant en a cinq, et les
    // empiler poussait le bouton — donc le local et le plan — sous le pli.
    let shown = bins.len().min(CARD_BIN_LIMIT);
    let hidden_bins = bins.len() - shown;
    // Quelque chose à voir dans la feuille : un point d'apport, un composteur, le local, ou
    // simplement les bacs que la carte n'a pas montrés. Un bouton vers une section vide
    // promettrait un local que l'hôte n'a pas renseigné.
    let elsewhere = !data.dropoff_points.is_empty()
        || !data.compost_location.trim().is_empty()
        || !data.bin_room_steps.is_empty()
        || hidden_bins > 0;

    if bins.is_empty() {
        // Le cas rural du §9 : pas de bacs, deux points d'apport et un composteur. Sans les
        // rangées, la carte n'aurait plus qu'un bouton — « touchez pour voir ce que je pourrais
        // afficher ». Elles prennent la place que les bacs n'occupent pas.
        children.extend(
            data.dropoff_points
                .iter()
                .map(|point| dropoff_row(data, point)),
        );
        children.extend(compost_row(data));
    } else {
        children.extend(bins.into_iter().take(shown));
        // « Où déposer mes déchets ? » plutôt que les points recopiés sous les bacs : la maquette
        // garde la carte courte et renvoie à la feuille, qui a le plan et les distances.
        //
        // Seulement quand il y a quelque chose à y voir — un bouton vers une section vide promet
        // un local ou un composteur que l'hôte n'a pas renseigné.
        if elsewhere {
            children.push(Component::Button(
                Button::new()
                    .label("i18n:guest.dropoff.cta")
                    // Sans icône : `Button` n'a pas le champ, et la maquette y met `map-pin`.
                    .variant(ButtonVariant::Outline)
                    .action(open()),
            ));
        }
    }

    Surface::new(
        Card::new()
            .icon(IconName::Recycle)
            .title("i18n:home.card.title")
            .action(open())
            .children(children),
    )
    .with_id(crate::guest::HOME_CARD)
}
