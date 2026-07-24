//! Guest fullscreen form surface opened from the formalities card.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use portaki_sdk::sdui::primitives::Text;

use super::empty::{empty_not_yet_state, empty_runtime_error_state, log_render_failure};
use super::home::build_form_surface;
use super::load::{load_guest_pre_arrival, GuestLoad};
use crate::config::load_config;

/// Fullscreen pre-arrival form (design page overlay).
#[portaki_sdk::surface(guest, id = "guest.form")]
pub fn render_guest_form(ctx: GuestContext) -> Surface {
    match render_form(&ctx) {
        Ok(surface) => surface,
        Err(error) => {
            log_render_failure(crate::ids::GUEST_FORM, &error);
            empty_runtime_error_state(crate::ids::GUEST_FORM)
        }
    }
}

fn render_form(ctx: &GuestContext) -> Result<Surface> {
    match load_guest_pre_arrival(ctx)? {
        GuestLoad::Empty(surface) => Ok(*surface),
        GuestLoad::NotYet => Ok(empty_not_yet_state(crate::ids::GUEST_FORM)),
        GuestLoad::Completed => Ok(Surface::new(
            Text::new()
                .text("i18n:home.card.thanks")
                .variant(TextVariant::Body),
        )
        .with_id(crate::ids::GUEST_FORM)),
        GuestLoad::Form => {
            let config = load_config().unwrap_or_default();
            Ok(build_form_surface(&config.questions))
        }
    }
}
