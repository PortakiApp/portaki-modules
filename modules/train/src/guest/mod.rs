//! Guest booklet surfaces. The SDK renders the inactive / incomplete / error states.

mod detail;
mod home;
mod item;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use detail::build_detail_page;
use home::{build_home_card, build_upcoming_card};
use item::build_item_page;

use portaki_sdk::sdui::primitives::EmptyState;

use crate::content::{departure_by_id, normalize_destination, normalize_direction};

/// Home card glance — mixed-destination departure board.
#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    Ok(build_home_card(&ctx))
}

/// Compact pre-arrival prep card rendered on the guest timeline (`role: upcoming`).
#[portaki_sdk::surface(
    guest,
    id = "upcoming.card",
    path = "upcoming",
    label_key = "nav.train",
    role = GuestRole::Upcoming
)]
pub fn render_upcoming_card(ctx: GuestContext) -> Result<Surface> {
    Ok(build_upcoming_card(&ctx))
}

/// Full train page (body-only — shell supplies header). `dest` arrives via route
/// params or the destination filter chips → `ctx.input.dest`.
#[portaki_sdk::surface(guest, id = "explore.detail", path = "train", label_key = "nav.train")]
pub fn render_explore_detail(ctx: GuestContext) -> Result<Surface> {
    let dest = ctx.input.get("dest").and_then(|value| value.as_str());
    let dir = ctx.input.get("dir").and_then(|value| value.as_str());
    Ok(build_detail_page(
        &ctx,
        normalize_direction(dir),
        normalize_destination(dest),
    ))
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
        .unwrap_or_default();
    match departure_by_id(wanted) {
        Some((destination, departure)) => Ok(build_item_page(destination, departure)),
        // Un horaire passé, un lien gardé en favori : la fiche le dit, elle ne montre pas le
        // train suivant comme si c'était celui qu'on cherchait.
        None => Ok(Surface::new(
            EmptyState::new()
                .title("i18n:explore.item.gone.title")
                .description("i18n:explore.item.gone.description")
                .icon(IconName::Train),
        )
        .with_id(EXPLORE_ITEM)),
    }
}
