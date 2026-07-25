//! Guest booklet surfaces.

mod empty;
mod form;
mod home;
mod load;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use empty::{empty_runtime_error_state, log_render_failure};
use home::build_home_card;
use load::{load_guest_reports, GuestLoad};

pub use form::render_guest_form;

/// Guest home card — teaser + open form overlay.
#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Surface {
    match render_with_data(&ctx) {
        Ok(surface) => surface,
        Err(error) => {
            log_render_failure(crate::ids::HOME_CARD, &error);
            empty_runtime_error_state(crate::ids::HOME_CARD)
        }
    }
}

fn render_with_data(ctx: &GuestContext) -> Result<Surface> {
    match load_guest_reports(ctx)? {
        GuestLoad::Empty(surface) => Ok(*surface),
        GuestLoad::Ready(reports) => Ok(build_home_card(&reports)),
    }
}
