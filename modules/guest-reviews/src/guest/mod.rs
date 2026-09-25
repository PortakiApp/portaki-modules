//! Guest booklet surfaces (inline post-stay — no overlay). The SDK's guest shell renders the
//! inactive, incomplete and error states.

mod home;
mod load;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, EmptyState, Text};
use portaki_sdk::sdui::surface::Surface;

use home::build_home_card;
use load::load_guest_data;

/// Shared renderer for the home card and the end-of-stay (post-stay) card.
///
/// Both surfaces show the same review content; the home card waits for the arrival, the
/// post-stay screen needs no such check.
fn render_card(ctx: &GuestContext, surface_id: SurfaceId, post_stay: bool) -> Result<Surface> {
    Ok(match load_guest_data(ctx)? {
        None => no_review_channel(surface_id),
        // The post-stay screen only opens once the stay is over.
        Some(_) if !post_stay && !crate::commands::has_arrived(ctx)? => not_yet(surface_id),
        Some(data) => build_home_card(&data),
    })
}

/// Before arrival there is no stay to review yet: the card says when it opens.
fn not_yet(surface_id: SurfaceId) -> Surface {
    Surface::new(
        Card::new()
            .icon(IconName::Star)
            .title("i18n:home.card.title")
            .child(
                Text::new()
                    .text("i18n:guest.notYet")
                    .variant(TextVariant::Body),
            ),
    )
    .with_id(surface_id)
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
    render_card(&ctx, HOME_CARD, false)
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
    render_card(&ctx, POST_STAY_CARD, true)
}
