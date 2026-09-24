//! Guest empty / fallback surfaces.

use portaki_sdk::host::{log, module};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{EmptyState, Text};
use portaki_sdk::sdui::surface::Surface;

pub fn empty_content_state(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:guest.empty.title")
            .description("i18n:guest.empty.description")
            .icon(IconName::Phone)
            .child(
                Text::new()
                    .text("i18n:guest.empty.hint")
                    .variant(TextVariant::Body),
            ),
    )
    .with_id(surface_id)
}

pub fn empty_runtime_error_state(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:guest.error.title")
            .description("i18n:guest.error.description")
            .icon(IconName::Phone)
            .child(
                Text::new()
                    .text("i18n:guest.error.hint")
                    .variant(TextVariant::Body),
            ),
    )
    .with_id(surface_id)
}

fn empty_config_state(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:module.status.incomplete.title")
            .description("i18n:module.status.incomplete.description")
            .icon(IconName::Sliders)
            .child(
                Text::new()
                    .text("i18n:module.status.incomplete.hint")
                    .variant(TextVariant::Body),
            ),
    )
    .with_id(surface_id)
}

fn empty_inactive_state(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:module.status.inactive.title")
            .description("i18n:module.status.inactive.description")
            .icon(IconName::Phone),
    )
    .with_id(surface_id)
}

pub fn log_render_failure(surface_id: SurfaceId, error: &PortakiError) {
    let mut fields = log::Fields::new();
    fields.insert("surfaceId", &surface_id);
    fields.insert("error", &error.to_string());
    let _ = log::error("emergency_contacts_guest_render_failed", &fields);
}

pub fn empty_state_if_module_not_ready(surface_id: SurfaceId) -> Result<Option<Surface>> {
    let status = module::status()?;
    if !status.workspace_enabled || !status.active {
        return Ok(Some(empty_inactive_state(surface_id)));
    }
    if status.incomplete {
        return Ok(Some(empty_config_state(surface_id)));
    }
    Ok(None)
}
