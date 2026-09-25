//! Guest booklet surfaces.

mod body;
mod detail;
mod home;
mod load;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::EmptyState;
use portaki_sdk::sdui::surface::Surface;

use detail::build_detail_surface;
use home::{build_home_card, build_upcoming_card};
use load::load_guest_data;

#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    render_with_data(&ctx, HOME_CARD, build_home_card)
}

/// Compact pre-arrival prep card rendered on the guest timeline (`role: upcoming`).
#[portaki_sdk::surface(
    guest,
    id = "upcoming.card",
    path = "upcoming",
    label_key = "nav.events",
    role = GuestRole::Upcoming
)]
pub fn render_upcoming_card(ctx: GuestContext) -> Result<Surface> {
    render_with_data(&ctx, UPCOMING_CARD, build_upcoming_card)
}

#[portaki_sdk::surface(
    guest,
    id = "explore.detail",
    path = "events/detail",
    label_key = "nav.events"
)]
pub fn render_explore_detail(ctx: GuestContext) -> Result<Surface> {
    render_with_data(&ctx, EXPLORE_DETAIL, build_detail_surface)
}

fn render_with_data(
    ctx: &GuestContext,
    surface_id: SurfaceId,
    build: fn(&load::GuestData) -> Surface,
) -> Result<Surface> {
    Ok(match load_guest_data(ctx, surface_id)? {
        Some(data) => build(&data),
        None => nothing_planned(surface_id),
    })
}

/// Nothing to show yet — written for the guest, who can do nothing about it.
fn nothing_planned(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:guest.empty.title")
            .description("i18n:guest.empty.description")
            .icon(IconName::Calendar),
    )
    .with_id(surface_id)
}
