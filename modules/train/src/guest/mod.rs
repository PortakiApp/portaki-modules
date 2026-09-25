//! Guest booklet surfaces. The SDK renders the inactive / incomplete / error states.

mod detail;
mod home;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use detail::build_detail_page;
use home::{build_home_card, build_upcoming_card};

use crate::content::normalize_destination;

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
    Ok(build_detail_page(&ctx, normalize_destination(dest)))
}
