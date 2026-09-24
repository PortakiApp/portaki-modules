//! Guest booklet surfaces.

pub(crate) mod depart;
mod empty;
mod home;
mod load;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use empty::{empty_not_yet_card, empty_runtime_error_state, log_render_failure};
use home::build_home_card;
use load::{load_guest_checklist, GuestLoad};

/// Guest home card — the open guest lists, progress + inline toggles (no overlay).
#[portaki_sdk::surface(
    guest,
    id = "home.card",
    path = "checklist",
    label_key = "home.card.title"
)]
pub fn render_home_card(ctx: GuestContext) -> Surface {
    render_card(&ctx, crate::ids::HOME_CARD, false)
}

/// End-of-stay card — the departure lists, still tickable once the stay is over.
#[portaki_sdk::surface(
    guest,
    id = "post-stay.card",
    path = "post-stay",
    label_key = "home.card.title",
    role = "post-stay"
)]
pub fn render_post_stay_card(ctx: GuestContext) -> Surface {
    render_card(&ctx, crate::ids::POST_STAY_CARD, true)
}

fn render_card(ctx: &GuestContext, surface_id: SurfaceId, departure_only: bool) -> Surface {
    let rendered = load_guest_checklist(ctx, surface_id, departure_only).map(|load| match load {
        GuestLoad::Empty(surface) => *surface,
        GuestLoad::NotYet => empty_not_yet_card(surface_id),
        GuestLoad::Ready(data) => build_home_card(&data, surface_id),
    });
    rendered.unwrap_or_else(|error| {
        log_render_failure(surface_id, &error);
        empty_runtime_error_state(surface_id)
    })
}
