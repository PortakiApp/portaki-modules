//! Guest fullscreen form surface opened from the formalities card.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::EmptyState;
use portaki_sdk::sdui::surface::Surface;

use super::home::{build_form_surface, build_readonly_surface};
use super::load::{load_guest_pre_arrival, GuestLoad};
use crate::config::ModuleConfig;

/// Fullscreen pre-arrival form (design page overlay).
#[portaki_sdk::surface(
    guest,
    id = "guest.form",
    path = "pre-arrival-form/form",
    label_key = "nav.pre-arrival-form"
)]
pub fn render_guest_form(ctx: GuestContext) -> Result<Surface> {
    match load_guest_pre_arrival(&ctx)? {
        GuestLoad::NotYet => Ok(not_yet(GUEST_FORM)),
        GuestLoad::Locked { response } => {
            let config = ModuleConfig::load(&ctx)?;
            Ok(build_readonly_surface(&config, &response))
        }
        GuestLoad::Form {
            completed,
            existing,
        } => {
            let config = ModuleConfig::load(&ctx)?;
            Ok(build_form_surface(&config, existing.as_ref(), completed))
        }
    }
}

/// Form gated by `show_when` — guest shell hides non-error EmptyStates (no teaser card).
fn not_yet(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:home.card.notYet")
            .description("i18n:home.card.notYet")
            .icon(IconName::ClipboardList),
    )
    .with_id(surface_id)
}
