//! Guest fullscreen form surface opened from the formalities card.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use super::empty::{empty_not_yet_state, empty_runtime_error_state, log_render_failure};
use super::home::{build_form_surface, build_readonly_surface};
use super::load::{load_guest_pre_arrival, GuestLoad};
use crate::config::ModuleConfig;

/// Fullscreen pre-arrival form (design page overlay).
#[portaki_sdk::surface(
    guest,
    id = "guest.form",
    path = "pre-arrival-form/form",
    label_key = "nav.pre-arrival-form"
)]
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
        GuestLoad::Locked { response } => {
            let config = ModuleConfig::read(ctx)?;
            Ok(build_readonly_surface(&config, &response))
        }
        GuestLoad::Form {
            completed,
            existing,
        } => {
            let config = ModuleConfig::read(ctx)?;
            Ok(build_form_surface(&config, existing.as_ref(), completed))
        }
    }
}
