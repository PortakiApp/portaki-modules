//! Guest booklet surfaces.

mod empty;
mod home;
mod load;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use empty::{empty_not_yet_state, empty_runtime_error_state, log_render_failure};
use home::{build_completed_card, build_form_card};
use load::{load_guest_pre_arrival, GuestLoad};

use crate::config::load_config;

/// Guest home card — short pre-arrival form (inline, no overlay).
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
    match load_guest_pre_arrival(ctx)? {
        GuestLoad::Empty(surface) => Ok(*surface),
        GuestLoad::NotYet => Ok(empty_not_yet_state(crate::ids::HOME_CARD)),
        GuestLoad::Form => {
            let config = load_config().unwrap_or_default();
            Ok(build_form_card(&config.questions))
        }
        GuestLoad::Completed => Ok(build_completed_card()),
    }
}
