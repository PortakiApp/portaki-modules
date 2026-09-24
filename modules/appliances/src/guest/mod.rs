//! Guest booklet surfaces.

mod empty;
mod home;
mod item;
mod page;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use empty::{empty_runtime_error_state, log_render_failure};
use home::build_home_card;
use item::build_item_detail;
use page::build_detail_page;

use crate::content::AppliancesPayload;
use crate::queries::load_payload;

#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Surface {
    match render_with_payload(&ctx, crate::ids::HOME_CARD, build_home_card) {
        Ok(surface) => surface,
        Err(error) => {
            log_render_failure(crate::ids::HOME_CARD, &error);
            empty_runtime_error_state(crate::ids::HOME_CARD)
        }
    }
}

#[portaki_sdk::surface(
    guest,
    id = "explore.detail",
    path = "appliances",
    label_key = "nav.appliances"
)]
pub fn render_explore_detail(ctx: GuestContext) -> Surface {
    match render_with_payload(&ctx, crate::ids::EXPLORE_DETAIL, build_detail_page) {
        Ok(surface) => surface,
        Err(error) => {
            log_render_failure(crate::ids::EXPLORE_DETAIL, &error);
            empty_runtime_error_state(crate::ids::EXPLORE_DETAIL)
        }
    }
}

/// Device detail. `deviceId` arrives via guest route params → render `input` → `ctx.input`.
#[portaki_sdk::surface(
    guest,
    id = "explore.item",
    path = "appliances/:deviceId",
    label_key = "nav.appliance"
)]
pub fn render_explore_item(ctx: GuestContext) -> Surface {
    let device_id = ctx
        .input
        .get("deviceId")
        .and_then(|value| value.as_str())
        .map(str::to_string);
    match load_for_item(&ctx, device_id.as_deref()) {
        Ok(surface) => surface,
        Err(error) => {
            log_render_failure(crate::ids::EXPLORE_ITEM, &error);
            empty_runtime_error_state(crate::ids::EXPLORE_ITEM)
        }
    }
}

fn render_with_payload(
    ctx: &GuestContext,
    surface_id: SurfaceId,
    build: fn(&AppliancesPayload) -> Surface,
) -> Result<Surface> {
    if let Some(surface) = empty::empty_state_if_module_not_ready(surface_id)? {
        return Ok(surface);
    }
    let payload = load_payload(ctx)?;
    if payload.is_empty_for_guest() {
        return Ok(empty::empty_content_state(surface_id));
    }
    Ok(build(&payload))
}

fn load_for_item(ctx: &GuestContext, device_id: Option<&str>) -> Result<Surface> {
    if let Some(surface) = empty::empty_state_if_module_not_ready(crate::ids::EXPLORE_ITEM)? {
        return Ok(surface);
    }
    let payload = load_payload(ctx)?;
    if payload.is_empty_for_guest() {
        return Ok(empty::empty_content_state(crate::ids::EXPLORE_ITEM));
    }
    Ok(build_item_detail(&payload, device_id))
}
