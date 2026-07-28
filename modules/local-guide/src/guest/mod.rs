//! Guest booklet surfaces.

mod body;
mod detail;
mod empty;
mod home;
mod load;
mod upcoming;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use detail::build_detail_surface;
use empty::{empty_runtime_error_state, log_render_failure};
use home::build_home_card;
use load::{load_guest_data, GuestLoad};
use upcoming::build_upcoming_card;

#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Surface {
    match render_with_data(&ctx, crate::ids::HOME_CARD, build_home_card) {
        Ok(surface) => surface,
        Err(error) => {
            log_render_failure(crate::ids::HOME_CARD, &error);
            empty_runtime_error_state(crate::ids::HOME_CARD)
        }
    }
}

#[portaki_sdk::surface(guest, id = "upcoming.card")]
pub fn render_upcoming_card(ctx: GuestContext) -> Surface {
    match render_with_data(&ctx, crate::ids::UPCOMING_CARD, build_upcoming_card) {
        Ok(surface) => surface,
        Err(error) => {
            log_render_failure(crate::ids::UPCOMING_CARD, &error);
            empty_runtime_error_state(crate::ids::UPCOMING_CARD)
        }
    }
}

#[portaki_sdk::surface(guest, id = "explore.detail")]
pub fn render_explore_detail(ctx: GuestContext) -> Surface {
    match render_with_data(&ctx, crate::ids::EXPLORE_DETAIL, build_detail_surface) {
        Ok(surface) => surface,
        Err(error) => {
            log_render_failure(crate::ids::EXPLORE_DETAIL, &error);
            empty_runtime_error_state(crate::ids::EXPLORE_DETAIL)
        }
    }
}

fn render_with_data(
    ctx: &GuestContext,
    surface_id: SurfaceId,
    build: fn(&load::GuestData) -> Surface,
) -> Result<Surface> {
    match load_guest_data(ctx, surface_id)? {
        GuestLoad::Empty(surface) => Ok(*surface),
        GuestLoad::Ready(data) => Ok(build(&data)),
    }
}
