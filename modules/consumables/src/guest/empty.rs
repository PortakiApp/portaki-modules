//! Guest empty / fallback surfaces.

use portaki_sdk::host::{log, module};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, EmptyState, Text};
use portaki_sdk::sdui::surface::Surface;

pub fn empty_no_items_card(surface_id: SurfaceId) -> Surface {
    Surface::new(
        Card::new()
            .icon("package")
            .title("i18n:home.card.title")
            .child(
                Text::new()
                    .text("i18n:home.card.empty")
                    .variant(TextVariant::Body),
            ),
    )
    .with_id(surface_id)
}

pub fn empty_runtime_error_state(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:home.card.error.title")
            .description("i18n:home.card.error.description")
            .icon("package")
            .child(
                Text::new()
                    .text("i18n:home.card.unavailable")
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
            .icon("sliders")
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
            .icon("package"),
    )
    .with_id(surface_id)
}

pub fn log_render_failure(surface_id: SurfaceId, error: &PortakiError) {
    let mut fields = log::Fields::new();
    fields.insert("surfaceId", &surface_id);
    fields.insert("error", &error.to_string());
    let _ = log::error("consumables_guest_render_failed", &fields);
}

/// Returns an EmptyState when the property-module is not ready to serve guest content.
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
