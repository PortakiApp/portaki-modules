//! Load current + forecast for guest surfaces.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::surface::Surface;

use crate::config::ModuleConfig;
use crate::entities::WeatherUnits;
use crate::queries::{get_current, get_forecast, GetCurrentArgs, GetForecastArgs};
use crate::weather::{has_open_weather, resolve_city_label, WeatherCurrent, WeatherForecast};

use super::empty::{empty_capability_state, empty_state_if_module_not_ready};

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

/// Shared gate + fetch for home card and explore sheet.
pub fn load_guest_weather(ctx: &GuestContext, surface_id: SurfaceId) -> Result<GuestLoad> {
    if let Some(surface) = empty_state_if_module_not_ready(surface_id)? {
        return Ok(GuestLoad::Empty(Box::new(surface)));
    }
    if !has_open_weather(ctx) {
        return Ok(GuestLoad::Empty(Box::new(empty_capability_state(
            surface_id,
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
