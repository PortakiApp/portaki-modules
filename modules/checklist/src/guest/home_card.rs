//! Guest home card — the open guest lists, progress + inline toggles (no overlay).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use super::card::render_card;

/// Guest home card — the open guest lists, progress + inline toggles (no overlay).
#[portaki_sdk::surface(
    guest,
    id = "home.card",
    path = "checklist",
    label_key = "home.card.title"
)]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    render_card(&ctx, HOME_CARD, false)
}
