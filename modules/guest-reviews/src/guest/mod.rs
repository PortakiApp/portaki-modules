//! Guest booklet surfaces (inline post-stay — no overlay).

mod empty;
mod home;
mod load;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use empty::{empty_runtime_error_state, log_render_failure};
use home::build_home_card;
use load::{load_guest_data, GuestLoad};

/// Shared renderer for the home card and the end-of-stay (post-stay) card.
///
/// Both surfaces show the same review content — only the surface id used for
/// data loading / error reporting differs.
fn render_card(ctx: &GuestContext, surface_id: SurfaceId) -> Surface {
    match load_guest_data(ctx, surface_id) {
        Ok(GuestLoad::Empty(surface)) => *surface,
        Ok(GuestLoad::Ready(data)) => build_home_card(&data),
        Err(error) => {
            log_render_failure(surface_id, &error);
            empty_runtime_error_state(surface_id)
        }
    }
}

#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Surface {
    render_card(&ctx, crate::ids::HOME_CARD)
}

/// End-of-stay card — same content as the home card, rendered on the dedicated
/// post-stay screen once the stay is over.
#[portaki_sdk::surface(guest, id = "post-stay.card")]
pub fn render_post_stay_card(ctx: GuestContext) -> Surface {
    render_card(&ctx, crate::ids::POST_STAY_CARD)
}
