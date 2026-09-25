//! Load stay reports for guest surfaces.

use portaki_sdk::prelude::*;

use crate::entities::LostFoundReport;
use crate::storage;

/// The stay's reports, oldest first; none outside a stay.
pub fn load_guest_reports(ctx: &GuestContext) -> Result<Vec<LostFoundReport>> {
    let Some(guest) = ctx.guest.as_ref() else {
        return Ok(Vec::new());
    };
    storage::list_by_stay(guest.session_id)
}
