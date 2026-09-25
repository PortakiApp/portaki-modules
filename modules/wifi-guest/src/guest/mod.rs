//! Guest booklet surfaces. The SDK's guest shell renders the inactive, incomplete and error
//! states; the module only says when there is nothing to show yet.

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
    render_with_data(&ctx, HOME_CARD, build_home_card)
}

#[portaki_sdk::surface(
    guest,
    id = "explore.detail",
    path = "wifi-guest/detail",
    label_key = "nav.wifi-guest"
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
        None => no_network(surface_id),
    })
}

/// No network yet: the required `ssid` makes the module incomplete, so the guest mostly meets
/// the SDK's state; this one covers a config the platform does not check (a first install).
fn no_network(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:guest.empty.title")
            .description("i18n:guest.empty.description")
            .icon(IconName::Wifi),
    )
    .with_id(surface_id)
}
