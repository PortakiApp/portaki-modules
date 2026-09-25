//! Guest booklet surfaces.

mod body;
mod detail;
mod home;
mod load;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::EmptyState;
use portaki_sdk::sdui::surface::Surface;

use detail::build_detail_surface;
use home::build_home_card;
use load::load_guest_data;

#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    render_with_data(&ctx, crate::ids::HOME_CARD, build_home_card)
}

#[portaki_sdk::surface(
    guest,
    id = "explore.detail",
    path = "facility-hours/detail",
    label_key = "nav.facility-hours"
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
        Some(data) => build(&data),
        None => empty_content_state(surface_id),
    })
}

/// Nothing to show yet — the SDK renders the inactive, incomplete and error states itself.
fn empty_content_state(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:guest.empty.title")
            .description("i18n:guest.empty.description")
            .icon(IconName::Clock),
    )
    .with_id(surface_id)
}
