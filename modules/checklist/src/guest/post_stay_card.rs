//! End-of-stay card — the departure lists, still tickable once the stay is over.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use super::card::render_card;

/// End-of-stay card — the departure lists, still tickable once the stay is over.
#[portaki_sdk::surface(
    guest,
    id = "post-stay.card",
    path = "checklist/post-stay",
    label_key = "home.card.title",
    role = GuestRole::PostStay
)]
pub fn render_post_stay_card(ctx: GuestContext) -> Result<Surface> {
    render_card(&ctx, POST_STAY_CARD, true)
}
