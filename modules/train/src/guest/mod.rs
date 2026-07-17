//! Guest booklet surfaces.

mod detail;
mod empty;
mod home;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use detail::build_detail_page;
use empty::{empty_runtime_error_state, empty_state_if_module_not_ready, log_render_failure};
use home::build_home_card;

use crate::content::normalize_destination;

/// Home card glance — mixed-destination departure board.
#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Surface {
    match render_home(&ctx) {
        Ok(surface) => surface,
        Err(error) => {
            log_render_failure("home.card", &error);
            empty_runtime_error_state("home.card")
        }
    }
}

/// Full train page (body-only — shell supplies header). `dest` arrives via route
/// params or the destination filter chips → `ctx.input.dest`.
#[portaki_sdk::surface(guest, id = "explore.detail")]
pub fn render_explore_detail(ctx: GuestContext) -> Surface {
    let dest = ctx.input.get("dest").and_then(|value| value.as_str());
    let selected = normalize_destination(dest);
    match render_detail(&ctx, selected) {
        Ok(surface) => surface,
        Err(error) => {
            log_render_failure("explore.detail", &error);
            empty_runtime_error_state("explore.detail")
        }
    }
}

fn render_home(ctx: &GuestContext) -> Result<Surface> {
    if let Some(surface) = empty_state_if_module_not_ready("home.card")? {
        return Ok(surface);
    }
    Ok(build_home_card(ctx))
}

fn render_detail(ctx: &GuestContext, selected: &str) -> Result<Surface> {
    if let Some(surface) = empty_state_if_module_not_ready("explore.detail")? {
        return Ok(surface);
    }
    Ok(build_detail_page(ctx, selected))
}
