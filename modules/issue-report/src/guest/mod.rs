//! Guest booklet surfaces. The SDK's guest shell renders the inactive, incomplete and error
//! states.

mod form;
mod home;
mod load;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use home::build_home_card;
use load::load_guest_reports;

pub use form::render_guest_form;

/// Guest home card — teaser + open form overlay.
#[portaki_sdk::surface(
    guest,
    id = "home.card",
    path = "issue-report",
    label_key = "nav.issue-report"
)]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    Ok(build_home_card(&load_guest_reports(&ctx)?))
}
