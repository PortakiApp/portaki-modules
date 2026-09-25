//! Guest booklet surfaces (inline post-stay — no overlay). The SDK's guest shell renders the
//! inactive, incomplete and error states.

mod home;
mod load;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::EmptyState;
use portaki_sdk::sdui::surface::Surface;

use home::build_home_card;
use load::load_guest_data;

/// Shared renderer for the home card and the end-of-stay (post-stay) card.
///
/// Both surfaces show the same review content — only the surface id used for
/// data loading differs.
fn render_card(ctx: &GuestContext, surface_id: SurfaceId) -> Result<Surface> {
    Ok(match load_guest_data(ctx)? {
        Some(data) => build_home_card(&data),
        None => no_review_channel(surface_id),
    })
}

/// No platform the guest can use for this stay (none selected, or Airbnb without its link or
/// not the booking channel).
fn no_review_channel(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:guest.empty.title")
            .description("i18n:guest.empty.description")
            .icon(IconName::Star),
    )
    .with_id(surface_id)
}

#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    render_card(&ctx, crate::ids::HOME_CARD)
}

/// End-of-stay card — same content as the home card, rendered on the dedicated
/// post-stay screen once the stay is over.
#[portaki_sdk::surface(
    guest,
    id = "post-stay.card",
    path = "post-stay",
    label_key = "nav.guest-reviews",
    role = GuestRole::PostStay
)]
pub fn render_post_stay_card(ctx: GuestContext) -> Result<Surface> {
    render_card(&ctx, crate::ids::POST_STAY_CARD)
}
