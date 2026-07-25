//! Guest booklet surfaces.

mod empty;
mod home;
mod load;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use empty::{empty_not_yet_card, empty_runtime_error_state, log_render_failure};
use home::build_home_card;
use load::{load_guest_checklist, GuestLoad};

/// Guest home card — progress + inline toggles (no overlay).
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
    match load_guest_checklist(ctx)? {
        GuestLoad::Empty(surface) => Ok(*surface),
        GuestLoad::NotYet => Ok(empty_not_yet_card(crate::ids::HOME_CARD)),
        GuestLoad::Ready(data) => Ok(build_home_card(&data)),
    }
}
