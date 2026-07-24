//! Load pre-arrival status for guest surfaces.

use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use super::empty::empty_state_if_module_not_ready;
use crate::config::load_config;
use crate::show_when::is_form_available;
use crate::storage;

pub enum GuestLoad {
    Empty(Box<Surface>),
    NotYet,
    Form,
    Completed,
}

pub fn load_guest_pre_arrival(ctx: &GuestContext) -> Result<GuestLoad> {
    if let Some(surface) = empty_state_if_module_not_ready(crate::ids::HOME_CARD)? {
        return Ok(GuestLoad::Empty(Box::new(surface)));
    }

    let Some(guest) = ctx.guest.as_ref() else {
        return Ok(GuestLoad::Form);
    };

    if storage::find_by_stay(guest.session_id)?.is_some() {
        return Ok(GuestLoad::Completed);
    }

    let config = load_config().unwrap_or_default();
    let checkin_at = ctx.stay.as_ref().and_then(|stay| stay.checkin_at);
    let now = time::now().unwrap_or_else(|_| chrono::Utc::now());
    if !is_form_available(config.show_when, now, checkin_at) {
        return Ok(GuestLoad::NotYet);
    }

    Ok(GuestLoad::Form)
}
