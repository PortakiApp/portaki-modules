//! Guest booklet surfaces. The SDK renders the inactive / incomplete / error states.

mod home;
mod page;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::EmptyState;
use portaki_sdk::sdui::surface::Surface;

use home::build_home_card;
use page::build_detail_page;

use crate::content::RulesPayload;
use crate::queries::load_payload;

/// Guest home booklet card — glance of first rules.
#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    render_with_payload(&ctx, crate::ids::HOME_CARD, build_home_card)
}

/// Full rules page (body-only — shell supplies header).
#[portaki_sdk::surface(guest, id = "explore.detail", path = "rules", label_key = "nav.rules")]
pub fn render_explore_detail(ctx: GuestContext) -> Result<Surface> {
    render_with_payload(&ctx, crate::ids::EXPLORE_DETAIL, build_detail_page)
}

fn render_with_payload(
    ctx: &GuestContext,
    surface_id: SurfaceId,
    build: fn(&RulesPayload) -> Surface,
) -> Result<Surface> {
    let payload = load_payload(ctx)?;
    if payload.is_empty() {
        return Ok(no_rules_state(surface_id));
    }
    Ok(build(&payload))
}

/// No rule written yet.
fn no_rules_state(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:home.card.empty.title")
            .description("i18n:home.card.empty.description")
            .icon(IconName::Scale),
    )
    .with_id(surface_id)
}
