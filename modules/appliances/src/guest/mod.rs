//! Guest booklet surfaces. The SDK renders the inactive / incomplete / error states.

mod home;
mod item;
mod page;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::EmptyState;
use portaki_sdk::sdui::surface::Surface;

use home::build_home_card;
use item::build_item_detail;
use page::build_detail_page;

use crate::content::AppliancesPayload;
use crate::queries::load_payload;

#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    render_with_payload(&ctx, crate::ids::HOME_CARD, build_home_card)
}

#[portaki_sdk::surface(
    guest,
    id = "explore.detail",
    path = "appliances",
    label_key = "nav.appliances"
)]
pub fn render_explore_detail(ctx: GuestContext) -> Result<Surface> {
    render_with_payload(&ctx, crate::ids::EXPLORE_DETAIL, build_detail_page)
}

/// Device detail. `deviceId` arrives via guest route params → render `input` → `ctx.input`.
#[portaki_sdk::surface(
    guest,
    id = "explore.item",
    path = "appliances/:deviceId",
    label_key = "nav.appliance"
)]
pub fn render_explore_item(ctx: GuestContext) -> Result<Surface> {
    let device_id = ctx
        .input
        .get("deviceId")
        .and_then(|value| value.as_str())
        .map(str::to_string);
    load_for_item(&ctx, device_id.as_deref())
}

fn render_with_payload(
    ctx: &GuestContext,
    surface_id: SurfaceId,
    build: fn(&AppliancesPayload) -> Surface,
) -> Result<Surface> {
    let payload = load_payload(ctx)?;
    if payload.is_empty_for_guest() {
        return Ok(no_appliances_state(surface_id));
    }
    Ok(build(&payload))
}

fn load_for_item(ctx: &GuestContext, device_id: Option<&str>) -> Result<Surface> {
    let payload = load_payload(ctx)?;
    if payload.is_empty_for_guest() {
        return Ok(no_appliances_state(crate::ids::EXPLORE_ITEM));
    }
    Ok(build_item_detail(&payload, device_id))
}

/// No appliance published yet.
fn no_appliances_state(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:home.card.empty.title")
            .description("i18n:home.card.empty.description")
            .icon(IconName::Plug),
    )
    .with_id(surface_id)
}
