//! Guest booklet surfaces.

mod body;
mod detail;
mod home;
mod item;
mod load;
mod upcoming;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::EmptyState;
use portaki_sdk::sdui::surface::Surface;

use detail::build_detail_surface;
use home::build_home_card;
use item::build_spot_item;
use load::load_guest_data;
use upcoming::build_upcoming_card;

#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    render_with_data(&ctx, HOME_CARD, build_home_card)
}

#[portaki_sdk::surface(
    guest,
    id = "upcoming.card",
    path = "upcoming",
    label_key = "nav.local-guide",
    role = GuestRole::Upcoming
)]
pub fn render_upcoming_card(ctx: GuestContext) -> Result<Surface> {
    render_with_data(&ctx, UPCOMING_CARD, build_upcoming_card)
}

#[portaki_sdk::surface(
    guest,
    id = "explore.detail",
    path = "local-guide/detail",
    label_key = "nav.local-guide"
)]
pub fn render_explore_detail(ctx: GuestContext) -> Result<Surface> {
    render_with_data(&ctx, EXPLORE_DETAIL, build_detail_surface)
}

fn render_with_data(
    ctx: &GuestContext,
    surface_id: SurfaceId,
    build: fn(&load::GuestData) -> Surface,
) -> Result<Surface> {
    Ok(match load_guest_data(ctx)? {
        Some(data) => build(&data),
        None => nothing_to_share(surface_id),
    })
}

/// Rien à montrer encore — écrit pour le voyageur, qui n'y peut rien.
fn nothing_to_share(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:guest.empty.title")
            .description("i18n:guest.empty.description")
            .icon(IconName::MapPin),
    )
    .with_id(surface_id)
}

/// La fiche d'une adresse. `spotId` arrive par les paramètres de route du livret.
#[portaki_sdk::surface(
    guest,
    id = "explore.item",
    path = "local-guide/detail/:spotId",
    label_key = "nav.spot"
)]
pub fn render_explore_item(ctx: GuestContext) -> Result<Surface> {
    let wanted = ctx
        .input
        .get("spotId")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();
    let Some(data) = load_guest_data(&ctx)? else {
        return Ok(nothing_to_share(EXPLORE_ITEM));
    };
    let found = data
        .spots
        .iter()
        .enumerate()
        .find(|(index, spot)| spot.route_id(*index) == wanted)
        .map(|(_, spot)| spot.clone());
    match found {
        Some(spot) => Ok(build_spot_item(&data, &spot)),
        None => Ok(nothing_to_share(EXPLORE_ITEM)),
    }
}
