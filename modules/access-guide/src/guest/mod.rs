//! Guest booklet surfaces.

mod body;
mod detail;
mod home;
mod load;
mod status_cell;
mod upcoming;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{EmptyState, Stack};
use portaki_sdk::sdui::surface::Surface;

use detail::build_detail_surface;
use home::build_home_card;
use load::{load_guest_data, GuestLoad};
use status_cell::build_status_cell;
use upcoming::build_upcoming_card;

#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    render_with_data(&ctx, HOME_CARD, build_home_card)
}

#[portaki_sdk::surface(
    guest,
    id = "upcoming.card",
    path = "upcoming",
    label_key = "nav.access-guide",
    role = GuestRole::Upcoming
)]
pub fn render_upcoming_card(ctx: GuestContext) -> Result<Surface> {
    render_with_data(&ctx, UPCOMING_CARD, build_upcoming_card)
}

#[portaki_sdk::surface(
    guest,
    id = "explore.detail",
    path = "access-guide/detail",
    label_key = "nav.access-guide"
)]
pub fn render_explore_detail(ctx: GuestContext) -> Result<Surface> {
    render_with_data(&ctx, EXPLORE_DETAIL, build_detail_surface)
}

/// The access cell of the status strip (§1.5): what the guest opens the door with, on the
/// schedule this module decides. The booklet draws it above the sections without knowing who
/// filled it.
///
/// It carries a `path` although it is not a page anyone navigates to. `portaki build` writes a
/// guest surface into the catalogue's `guestSurfaces` — the list the platform parses and the
/// booklet routes on — only when it declares one (`manifest/catalog.rs`, the `else if let
/// Some(path)` arm). A pathless surface is dropped with its role, so a cell that named no path
/// would reach the booklet as nothing at all. Until the catalogue carries a role without a
/// path, the address is the price of the role.
///
/// The host has written nothing: an empty tree, no `KeyValue`. The strip then falls back to a
/// grid of one or two cells rather than showing a cell with nothing in it — an unconfigured
/// module is the host's silence, not content, and silence does not render (§0.5).
#[portaki_sdk::surface(
    guest,
    id = "status.cell",
    path = "access-guide/status",
    label_key = "nav.access-guide",
    role = GuestRole::StatusCell
)]
pub fn render_status_cell(ctx: GuestContext) -> Result<Surface> {
    Ok(match load_guest_data(&ctx)? {
        GuestLoad::Empty => Surface::new(Stack::new()).with_id(STATUS_CELL),
        GuestLoad::Ready(data) => build_status_cell(&data),
    })
}

fn render_with_data(
    ctx: &GuestContext,
    surface_id: SurfaceId,
    build: fn(&load::GuestData) -> Surface,
) -> Result<Surface> {
    Ok(match load_guest_data(ctx)? {
        GuestLoad::Empty => no_instructions_yet(surface_id),
        GuestLoad::Ready(data) => build(&data),
    })
}

/// The host has written nothing yet: tell the guest, not the host.
fn no_instructions_yet(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:guest.empty.title")
            .description("i18n:guest.empty.description")
            .icon(IconName::Car),
    )
    .with_id(surface_id)
}
