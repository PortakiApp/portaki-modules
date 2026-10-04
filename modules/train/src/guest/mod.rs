//! Les surfaces du livret. Le SDK rend les états inactif / incomplet / erreur.

mod detail;
mod home;
mod item;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::EmptyState;
use portaki_sdk::sdui::surface::Surface;

use detail::{build_detail_page, build_error_page};
use home::{build_home_card, build_upcoming_card};
use item::build_item_page;

use crate::board::{self, BoardView, ALL_STATIONS};
use crate::sncf::Way;

/// La destination demandée par l'adresse, ou toutes.
fn wanted_destination(ctx: &GuestContext) -> String {
    ctx.input
        .get("dest")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(ALL_STATIONS)
        .to_string()
}

fn wanted_way(ctx: &GuestContext) -> Way {
    Way::parse(ctx.input.get("dir").and_then(|value| value.as_str()))
}

/// Le tableau, ou la surface qui dit pourquoi il n'y en a pas.
///
/// Chaque surface passe par là : une carte d'accueil qui montrerait un tableau vide sans rien dire
/// ferait croire qu'il n'y a pas de train, là où c'est la clé ou la gare qui manque.
fn with_board(
    ctx: &GuestContext,
    way: Way,
    destination: &str,
    render: impl FnOnce(&BoardView) -> Surface,
) -> Surface {
    match board::load(ctx, way, destination) {
        Ok(view) => render(&view),
        Err(error) => build_error_page(&error),
    }
}

/// Carte d'accueil — le tableau en aperçu.
#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    Ok(with_board(&ctx, Way::From, ALL_STATIONS, build_home_card))
}

/// La carte compacte d'avant l'arrivée, sur la frise du livret (`role: upcoming`).
#[portaki_sdk::surface(
    guest,
    id = "upcoming.card",
    path = "upcoming",
    label_key = "nav.train",
    role = GuestRole::Upcoming
)]
pub fn render_upcoming_card(ctx: GuestContext) -> Result<Surface> {
    Ok(with_board(
        &ctx,
        Way::From,
        ALL_STATIONS,
        build_upcoming_card,
    ))
}

/// La page complète. `dir` et `dest` arrivent des paramètres de route ou des contrôles de la page.
#[portaki_sdk::surface(guest, id = "explore.detail", path = "train", label_key = "nav.train")]
pub fn render_explore_detail(ctx: GuestContext) -> Result<Surface> {
    let way = wanted_way(&ctx);
    let destination = wanted_destination(&ctx);
    Ok(with_board(&ctx, way, &destination, |view| {
        build_detail_page(view, way, &destination)
    }))
}

/// La fiche d'un départ. `departureId` arrive par les paramètres de route du livret.
#[portaki_sdk::surface(
    guest,
    id = "explore.item",
    path = "train/:departureId",
    label_key = "nav.departure"
)]
pub fn render_explore_item(ctx: GuestContext) -> Result<Surface> {
    let wanted = ctx
        .input
        .get("departureId")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();
    let way = wanted_way(&ctx);
    Ok(with_board(&ctx, way, ALL_STATIONS, |view| {
        match board::stop_by_route_id(view, &wanted) {
            Some(stop) => build_item_page(view, &stop),
            // Un horaire passé, un lien gardé en favori : la fiche le dit, elle ne montre pas le
            // train suivant comme si c'était celui qu'on cherchait.
            None => Surface::new(
                EmptyState::new()
                    .title("i18n:explore.item.gone.title")
                    .description("i18n:explore.item.gone.description")
                    .icon(IconName::Train),
            )
            .with_id(EXPLORE_ITEM),
        }
    }))
}
