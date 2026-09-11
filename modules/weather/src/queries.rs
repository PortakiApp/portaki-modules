//! Module queries — current conditions and multi-day forecast.

use chrono::{DateTime, Utc};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::cache;
use crate::config::load_config;
use crate::entities::WeatherUnits;
use crate::weather::{
    fetch_current_from_api, fetch_forecast_from_api, has_open_weather, map_current, map_forecast,
    WeatherCurrent, WeatherForecast,
};

/// Arguments for `getCurrent`.
#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCurrentArgs {
    /// Optional latitude override (defaults to property coordinates).
    pub lat: Option<f64>,
    /// Optional longitude override.
    pub lng: Option<f64>,
}

/// Arguments for `getForecast`.
#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetForecastArgs {
    /// Optional latitude override.
    pub lat: Option<f64>,
    /// Optional longitude override.
    pub lng: Option<f64>,
    /// Number of forecast days (default 5).
    pub days: Option<u8>,
}

struct QueryCoords {
    lat: f64,
    lng: f64,
    units: WeatherUnits,
    now: DateTime<Utc>,
}

fn resolve_coords(ctx: &Context, lat: Option<f64>, lng: Option<f64>) -> Result<QueryCoords> {
    if !has_open_weather(ctx) {
        return Err(PortakiError::CapabilityNotAvailable(
            "external.open-weather".to_string(),
        ));
    }
    let config = load_config()?;
    Ok(QueryCoords {
        lat: lat.unwrap_or(ctx.property.lat),
        lng: lng.unwrap_or(ctx.property.lng),
        units: config.units,
        now: time::now()?,
    })
}

fn fetch_pair(coords: &QueryCoords, days: u8) -> Result<(WeatherCurrent, WeatherForecast)> {
    let current = map_current(
        fetch_current_from_api(coords.lat, coords.lng)?,
        coords.units,
        coords.now,
    );
    let forecast = map_forecast(
        fetch_forecast_from_api(coords.lat, coords.lng, days)?,
        coords.units,
        coords.now,
    );
    if let Err(error) =
        cache::store_current(coords.lat, coords.lng, coords.units, &current, &forecast)
    {
        log_cache_failure("weather_cache_store_failed", coords.lat, coords.lng, &error);
    }
    Ok((current, forecast))
}

#[portaki_sdk::query(name = "getCurrent")]
pub fn get_current(ctx: Context, args: GetCurrentArgs) -> Result<WeatherCurrent> {
    let coords = resolve_coords(&ctx, args.lat, args.lng)?;

    match cache::read_current(coords.lat, coords.lng, coords.units, coords.now) {
        Ok(Some(cached)) => return Ok(cached),
        Ok(None) => {}
        Err(error) => {
            log_cache_failure("weather_cache_read_failed", coords.lat, coords.lng, &error)
        }
    }

    let (current, _) = fetch_pair(&coords, 5)?;
    Ok(current)
}

#[portaki_sdk::query(name = "getForecast")]
pub fn get_forecast(ctx: Context, args: GetForecastArgs) -> Result<WeatherForecast> {
    let coords = resolve_coords(&ctx, args.lat, args.lng)?;
    let days = args.days.unwrap_or(5);

    match cache::read_forecast(coords.lat, coords.lng, coords.units, coords.now) {
        Ok(Some(cached)) => return Ok(cached),
        Ok(None) => {}
        Err(error) => {
            log_cache_failure("weather_cache_read_failed", coords.lat, coords.lng, &error)
        }
    }

    let (_, forecast) = fetch_pair(&coords, days)?;
    Ok(forecast)
}

fn log_cache_failure(event: &str, lat: f64, lng: f64, error: &PortakiError) {
    let mut fields = portaki_sdk::host::log::Fields::new();
    fields.insert("lat", &lat);
    fields.insert("lng", &lng);
    fields.insert("error", &error.to_string());
    let _ = portaki_sdk::host::log::warn(event, &fields);
}
