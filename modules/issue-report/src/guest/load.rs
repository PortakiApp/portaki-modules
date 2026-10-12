//! Load stay reports for guest surfaces.

use portaki_sdk::prelude::*;

use crate::entities::{IssueReport, IssueReportEvent};
use crate::storage;

/// The stay's reports, newest first, and their history; nothing outside a stay.
pub fn load_guest_reports(ctx: &GuestContext) -> Result<(Vec<IssueReport>, Vec<IssueReportEvent>)> {
    let Some(guest) = ctx.guest.as_ref() else {
        return Ok((Vec::new(), Vec::new()));
    };
    Ok((
        storage::list_by_stay(guest.session_id)?,
        storage::events_for_stay(guest.session_id)?,
    ))
}
