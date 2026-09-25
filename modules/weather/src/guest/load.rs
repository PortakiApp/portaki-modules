//! Load current + forecast for guest surfaces.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use crate::config::ModuleConfig;
use crate::entities::WeatherUnits;
use crate::queries::{get_current, get_forecast, GetCurrentArgs, GetForecastArgs};
use crate::weather::{has_open_weather, resolve_city_label, WeatherCurrent, WeatherForecast};

use super::empty::no_weather;

pub struct GuestWeatherData {
    pub current: WeatherCurrent,
    pub forecast: WeatherForecast,
    pub units: WeatherUnits,
    pub city: Option<String>,
}

pub enum GuestLoad {
    Ready(Box<GuestWeatherData>),
    Empty(Box<Surface>),
}

/// Shared gate + fetch for the guest surfaces. No capability or no position: a content empty
/// state, and no network call.
pub fn load_guest_weather(ctx: &GuestContext, surface_id: SurfaceId) -> Result<GuestLoad> {
    if !has_open_weather(ctx) {
        return Ok(GuestLoad::Empty(Box::new(no_weather(
            surface_id,
            "i18n:guest.unavailable.description",
        ))));
    }
    if ctx.property.coordinates.is_none() {
        return Ok(GuestLoad::Empty(Box::new(no_weather(
            surface_id,
            "i18n:guest.noLocation.description",
        ))));
    }

    let config = ModuleConfig::load(ctx)?;
    let current = get_current(
        ctx.clone(),
        GetCurrentArgs {
            lat: None,
            lng: None,
        },
    )?;
    let forecast = get_forecast(
        ctx.clone(),
        GetForecastArgs {
            lat: None,
            lng: None,
            days: Some(5),
        },
    )?;
    let city = resolve_city_label(
        current
            .city_name
            .as_deref()
            .or(forecast.city_name.as_deref()),
        ctx.property.address.as_deref(),
    );

    Ok(GuestLoad::Ready(Box::new(GuestWeatherData {
        current,
        forecast,
        units: config.units,
        city,
    })))
}
