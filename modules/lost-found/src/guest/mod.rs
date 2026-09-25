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

/// Shared renderer for the home card and the end-of-stay (post-stay) card: same content.
fn render_card(ctx: &GuestContext) -> Result<Surface> {
    Ok(build_home_card(&load_guest_reports(ctx)?))
}

/// Guest home card — teaser + open form overlay.
#[portaki_sdk::surface(
    guest,
    id = "home.card",
    path = "lost-found",
    label_key = "nav.lost-found"
)]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    render_card(&ctx)
}

/// End-of-stay card — same content as the home card, rendered on the dedicated
/// post-stay screen once the stay is over.
#[portaki_sdk::surface(
    guest,
    id = "post-stay.card",
    path = "post-stay",
    label_key = "nav.lost-found",
    role = GuestRole::PostStay
)]
pub fn render_post_stay_card(ctx: GuestContext) -> Result<Surface> {
    render_card(&ctx)
}
