//! Guest booklet surfaces.

mod detail;
mod home;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::EmptyState;
use portaki_sdk::sdui::surface::Surface;

use crate::config::ModuleConfig;
use detail::build_detail_surface;
use home::build_home_card;

/// Carte d'accueil, section Aide — discrète, sans contenu ni ton d'alerte (§2.22).
#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    if config.parse_shutoffs().is_empty() {
        return Ok(empty_content_state(HOME_CARD));
    }
    Ok(build_home_card())
}

/// Le détail, en plein écran : il se lit sous stress, pas dans un tiroir.
#[portaki_sdk::surface(
    guest,
    id = "explore.detail",
    path = "safety-shutoffs/detail",
    label_key = "nav.safety-shutoffs"
)]
pub fn render_explore_detail(ctx: GuestContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    if config.parse_shutoffs().is_empty() {
        return Ok(empty_content_state(EXPLORE_DETAIL));
    }
    Ok(build_detail_surface(&config, &ctx))
}

/// Nothing to show yet — the SDK renders the inactive, incomplete and error states itself.
fn empty_content_state(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:guest.empty.title")
            .description("i18n:guest.empty.description")
            .icon(IconName::Sliders),
    )
    .with_id(surface_id)
}
