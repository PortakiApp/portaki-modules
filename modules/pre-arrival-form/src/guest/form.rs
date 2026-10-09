//! Guest fullscreen form surface opened from the formalities card.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::EmptyState;
use portaki_sdk::sdui::surface::Surface;

use super::home::{build_form_surface, build_readonly_surface, FormInputs};
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
            Ok(build_readonly_surface(&config, &response, &ctx))
        }
        GuestLoad::Form {
            completed,
            existing,
        } => {
            let config = ModuleConfig::load(&ctx)?;
            let stay = ctx.stay.as_ref();
            let slots = crate::slots::checkin_hour(
                stay.and_then(|stay| stay.checkin_at),
                &ctx.property.timezone,
            )
            .map(|hour| config.slots(hour))
            .unwrap_or_default();
            Ok(build_form_surface(&FormInputs {
                questions: &config,
                existing: existing.as_ref(),
                completed,
                slots,
                host_name: ctx
                    .host
                    .as_ref()
                    .map(|host| host.name.trim().to_string())
                    .unwrap_or_default(),
                party_size: stay.and_then(|stay| stay.party_size),
                ctx: &ctx,
            }))
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
