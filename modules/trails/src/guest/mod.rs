//! Guest booklet surfaces.

mod detail;
mod home;
mod load;
mod rows;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::EmptyState;
use portaki_sdk::sdui::surface::Surface;

use detail::{build_trail_detail, build_trails_page};
use home::build_home_card;
use load::load_guest_data;

/// Carte d'accueil, section Autour — pastilles par niveau et les deux premiers itinéraires.
#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    let data = load_guest_data(&ctx)?;
    if data.trails.is_empty() {
        return Ok(empty_content_state(HOME_CARD));
    }
    Ok(build_home_card(&data))
}

/// La liste, une section par niveau.
#[portaki_sdk::surface(
    guest,
    id = "explore.detail",
    path = "trails",
    label_key = "nav.trails"
)]
pub fn render_explore_detail(ctx: GuestContext) -> Result<Surface> {
    let data = load_guest_data(&ctx)?;
    if data.trails.is_empty() {
        return Ok(empty_content_state(EXPLORE_DETAIL));
    }
    Ok(build_trails_page(&data))
}

/// La fiche d'un itinéraire. `trailId` arrive par les paramètres de route du livret.
#[portaki_sdk::surface(
    guest,
    id = "explore.item",
    path = "trails/:trailId",
    label_key = "nav.trail"
)]
pub fn render_explore_item(ctx: GuestContext) -> Result<Surface> {
    let data = load_guest_data(&ctx)?;
    let wanted = ctx
        .input
        .get("trailId")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();
    let found = data
        .trails
        .iter()
        .enumerate()
        .find(|(index, trail)| trail.route_id(*index) == wanted);
    match found {
        Some((_, trail)) => Ok(build_trail_detail(&data, trail)),
        None => Ok(Surface::new(
            EmptyState::new()
                .title("i18n:guest.item.notFound")
                .description("i18n:guest.item.notFound.description")
                .icon(IconName::Mountain),
        )
        .with_id(EXPLORE_ITEM)),
    }
}

/// Nothing to show yet — the SDK renders the inactive, incomplete and error states itself.
fn empty_content_state(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:guest.empty.title")
            .description("i18n:guest.empty.description")
            .icon(IconName::Mountain),
    )
    .with_id(surface_id)
}
