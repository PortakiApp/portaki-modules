//! Load pre-arrival status for guest surfaces.

use portaki_sdk::host::time;
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;
use crate::entities::PreArrivalResponse;
use crate::show_when::{is_editable_until_checkin, is_form_available};
use crate::storage;

pub enum GuestLoad {
    NotYet,
    /// Form open for fill or edit (before check-in).
    Form {
        completed: bool,
        existing: Option<PreArrivalResponse>,
    },
    /// Submitted and past check-in — review only.
    Locked {
        response: PreArrivalResponse,
    },
}

pub fn load_guest_pre_arrival(ctx: &GuestContext) -> Result<GuestLoad> {
    let Some(guest) = ctx.guest.as_ref() else {
        return Ok(GuestLoad::Form {
            completed: false,
            existing: None,
        });
    };

    let checkin_at = ctx.stay.as_ref().and_then(|stay| stay.checkin_at);
    let now = time::now()?;
    let editable = is_editable_until_checkin(now, checkin_at);

    if let Some(response) = storage::find_by_stay(guest.session_id)? {
        if editable {
            return Ok(GuestLoad::Form {
                completed: true,
                existing: Some(response),
            });
        }
        return Ok(GuestLoad::Locked { response });
    }

    let config = ModuleConfig::load(ctx)?;
    if !is_form_available(config.show_when, now, checkin_at) {
        return Ok(GuestLoad::NotYet);
    }

    Ok(GuestLoad::Form {
        completed: false,
        existing: None,
    })
}
