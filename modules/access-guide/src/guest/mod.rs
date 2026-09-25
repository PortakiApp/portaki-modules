//! Guest booklet surfaces.

mod body;
mod detail;
mod home;
mod load;
mod upcoming;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::EmptyState;
use portaki_sdk::sdui::surface::Surface;

use detail::build_detail_surface;
use home::build_home_card;
use load::{load_guest_data, GuestLoad};
use upcoming::build_upcoming_card;

#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    render_with_data(&ctx, crate::ids::HOME_CARD, build_home_card)
}

#[portaki_sdk::surface(
    guest,
    id = "upcoming.card",
    path = "upcoming",
    label_key = "nav.access-guide",
    role = GuestRole::Upcoming
)]
pub fn render_upcoming_card(ctx: GuestContext) -> Result<Surface> {
    render_with_data(&ctx, crate::ids::UPCOMING_CARD, build_upcoming_card)
}

#[portaki_sdk::surface(
    guest,
    id = "explore.detail",
    path = "access-guide/detail",
    label_key = "nav.access-guide"
)]
pub fn render_explore_detail(ctx: GuestContext) -> Result<Surface> {
    render_with_data(&ctx, crate::ids::EXPLORE_DETAIL, build_detail_surface)
}

fn render_with_data(
    ctx: &GuestContext,
    surface_id: SurfaceId,
    build: fn(&load::GuestData) -> Surface,
) -> Result<Surface> {
    Ok(match load_guest_data(ctx)? {
        GuestLoad::Empty => no_instructions_yet(surface_id),
        GuestLoad::Ready(data) => build(&data),
    })
}

/// The host has written nothing yet: tell the guest, not the host.
fn no_instructions_yet(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:guest.empty.title")
            .description("i18n:guest.empty.description")
            .icon(IconName::Car),
    )
    .with_id(surface_id)
}
