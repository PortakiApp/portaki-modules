//! Guest booklet surfaces.

mod body;
mod components;
mod details;
mod empty;
mod home;
mod load;
mod sheet;
mod table;
mod upcoming;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use home::build_home_card;
use load::{load_guest_weather, GuestLoad};
use sheet::build_sheet_surface;
use upcoming::build_upcoming_card;

/// Guest home booklet card with current conditions.
#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    render_with_data(&ctx, HOME_CARD, build_home_card)
}

/// Compact pre-arrival prep card rendered on the guest timeline.
#[portaki_sdk::surface(
    guest,
    id = "upcoming.card",
    path = "upcoming",
    label_key = "nav.weather",
    role = GuestRole::Upcoming
)]
pub fn render_upcoming_card(ctx: GuestContext) -> Result<Surface> {
    render_with_data(&ctx, UPCOMING_CARD, build_upcoming_card)
}

/// Sheet / explore detail — same weather body as the card (design `block("weather")` in sheet).
#[portaki_sdk::surface(
    guest,
    id = "explore.forecast",
    path = "weather/forecast",
    label_key = "nav.forecast"
)]
pub fn render_explore_forecast(ctx: GuestContext) -> Result<Surface> {
    render_with_data(&ctx, EXPLORE_FORECAST, build_sheet_surface)
}

fn render_with_data(
    ctx: &GuestContext,
    surface_id: SurfaceId,
    build: fn(&load::GuestWeatherData) -> Surface,
) -> Result<Surface> {
    match load_guest_weather(ctx, surface_id)? {
        GuestLoad::Empty(surface) => Ok(*surface),
        GuestLoad::Ready(data) => Ok(build(&data)),
    }
}
