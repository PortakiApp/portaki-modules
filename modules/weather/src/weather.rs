//! Domain types and OpenWeather mapping helpers.

use chrono::{DateTime, Datelike, Utc};
use portaki_connectors::open_weather::{
    CurrentArgs, CurrentWeather, ForecastArgs, ForecastDay, ForecastResponse, OpenWeather,
};
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::entities::WeatherUnits;

/// TTL for current conditions cache (1 hour).
pub const CURRENT_CACHE_TTL_SECS: i64 = 3600;
/// TTL for forecast cache (6 hours).
pub const FORECAST_CACHE_TTL_SECS: i64 = 21_600;

/// Snapshot returned by `getCurrent`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeatherCurrent {
    pub temp_c: f64,
    pub condition: String,
    pub humidity: u8,
    pub uv_index: Option<f64>,
    pub description_key: String,
    pub units: WeatherUnits,
    pub fetched_at: DateTime<Utc>,
}

/// One forecast day for guests and queries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ForecastDayView {
    pub date: String,
    pub weekday_key: String,
    pub min_c: f64,
    pub max_c: f64,
    pub condition: String,
    pub display_temp_c: f64,
}

/// Forecast bundle returned by `getForecast`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeatherForecast {
    pub days: Vec<ForecastDayView>,
    pub units: WeatherUnits,
    pub fetched_at: DateTime<Utc>,
}

/// Serialized payload stored in `WeatherCache.current_json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedCurrentPayload {
    pub temp_c: f64,
    pub condition: String,
    pub humidity: u8,
    pub uv_index: Option<f64>,
}

/// Serialized payload stored in `WeatherCache.forecast_json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedForecastPayload {
    pub days: Vec<ForecastDayView>,
}

use std::sync::atomic::{AtomicUsize, Ordering};

/// Counts OpenWeather `current` connector calls in unit tests.
pub static CONNECTOR_CURRENT_CALLS: AtomicUsize = AtomicUsize::new(0);

/// Counts OpenWeather `forecast` connector calls in unit tests.
pub static CONNECTOR_FORECAST_CALLS: AtomicUsize = AtomicUsize::new(0);

/// Returns true when pool or BYOK OpenWeather capability is granted.
pub fn has_open_weather(ctx: &Context) -> bool {
    ctx.has_capability(capability::external::OPEN_WEATHER_POOL)
        || ctx.has_capability(capability::external::OPEN_WEATHER_BYOK)
}

/// Maps a condition code to an i18n description key.
pub fn description_key_for_condition(condition: &str) -> String {
    let normalized = condition.to_ascii_lowercase();
    if normalized.contains("clear") || normalized.contains("sun") {
        "weather.description.sunny".to_string()
    } else if normalized.contains("cloud") {
        "weather.description.cloudy".to_string()
    } else if normalized.contains("rain") || normalized.contains("drizzle") {
        "weather.description.rainy".to_string()
    } else if normalized.contains("snow") {
        "weather.description.snowy".to_string()
    } else if normalized.contains("storm") || normalized.contains("thunder") {
        "weather.description.stormy".to_string()
    } else if normalized.contains("fog") || normalized.contains("mist") {
        "weather.description.foggy".to_string()
    } else {
        "weather.description.cloudy".to_string()
    }
}

/// Maps OpenWeather condition → Lucide / pk-icon name (generic `Icon` primitive).
pub fn icon_name_for_condition(condition: &str) -> &'static str {
    let normalized = condition.to_ascii_lowercase();
    if normalized.contains("storm") || normalized.contains("thunder") {
        "cloud-lightning"
    } else if normalized.contains("snow") {
        "cloud-snow"
    } else if normalized.contains("rain") || normalized.contains("drizzle") {
        "cloud-rain"
    } else if normalized.contains("fog") || normalized.contains("mist") {
        "cloud-fog"
    } else if normalized.contains("clear") || normalized.contains("sun") {
        "sun"
    } else if normalized.contains("cloud") {
        "cloud-sun"
    } else {
        "cloud-sun"
    }
}

/// Formats a temperature for `Text` (hero omits unit letter — design shows `24°`).
pub fn format_temp_label(temp: f64, unit: &str, include_unit: bool) -> String {
    let rounded = temp.round() as i64;
    if !include_unit {
        return format!("{rounded}°");
    }
    let letter = match unit {
        "fahrenheit" | "F" | "f" => "F",
        _ => "C",
    };
    format!("{rounded}°{letter}")
}

/// UV badge i18n key from index.
pub fn uv_label_key(uv_index: f64) -> &'static str {
    if uv_index < 3.0 {
        "weather.uv.low"
    } else if uv_index < 6.0 {
        "weather.uv.moderate"
    } else if uv_index < 8.0 {
        "weather.uv.high"
    } else {
        "weather.uv.extreme"
    }
}

/// Whether to show the UV warning badge.
pub fn is_uv_high(uv_index: Option<f64>) -> bool {
    uv_index.is_some_and(|value| value >= 6.0)
}

/// Converts API current weather into module snapshot.
pub fn map_current(
    api: CurrentWeather,
    units: WeatherUnits,
    fetched_at: DateTime<Utc>,
) -> WeatherCurrent {
    let uv_index = estimate_uv_index(&api.condition);
    WeatherCurrent {
        temp_c: api.temp_c,
        condition: api.condition.clone(),
        humidity: api.humidity,
        uv_index,
        description_key: description_key_for_condition(&api.condition),
        units,
        fetched_at,
    }
}

/// Converts API forecast into guest-facing rows (max 5 days).
pub fn map_forecast(
    api: ForecastResponse,
    units: WeatherUnits,
    fetched_at: DateTime<Utc>,
) -> WeatherForecast {
    let days = api
        .days
        .into_iter()
        .take(5)
        .map(|day| map_forecast_day(day, units))
        .collect();
    WeatherForecast {
        days,
        units,
        fetched_at,
    }
}

fn map_forecast_day(day: ForecastDay, units: WeatherUnits) -> ForecastDayView {
    let display_temp_c = (day.min_c + day.max_c) / 2.0;
    ForecastDayView {
        date: day.date.clone(),
        weekday_key: weekday_key_for_date(&day.date),
        min_c: day.min_c,
        max_c: day.max_c,
        condition: day.condition,
        display_temp_c: convert_temp(display_temp_c, units),
    }
}

fn weekday_key_for_date(date: &str) -> String {
    if let Ok(parsed) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        let weekday = parsed.weekday().number_from_monday();
        match weekday {
            1 => "day.monday",
            2 => "day.tuesday",
            3 => "day.wednesday",
            4 => "day.thursday",
            5 => "day.friday",
            6 => "day.saturday",
            _ => "day.sunday",
        }
        .to_string()
    } else {
        "day.monday".to_string()
    }
}

/// Converts Celsius to the configured display unit.
pub fn convert_temp(value_c: f64, units: WeatherUnits) -> f64 {
    match units {
        WeatherUnits::Celsius => value_c,
        WeatherUnits::Fahrenheit => (value_c * 9.0 / 5.0) + 32.0,
    }
}

fn estimate_uv_index(condition: &str) -> Option<f64> {
    let normalized = condition.to_ascii_lowercase();
    if normalized.contains("clear") || normalized.contains("sun") {
        Some(7.5)
    } else if normalized.contains("cloud") {
        Some(4.0)
    } else {
        Some(2.0)
    }
}

/// Fetches current weather from the OpenWeather connector.
pub fn fetch_current_from_api(lat: f64, lng: f64) -> Result<CurrentWeather> {
    CONNECTOR_CURRENT_CALLS.fetch_add(1, Ordering::SeqCst);
    OpenWeather::current(&CurrentArgs { lat, lng })
}

/// Fetches forecast from the OpenWeather connector.
pub fn fetch_forecast_from_api(lat: f64, lng: f64, days: u8) -> Result<ForecastResponse> {
    CONNECTOR_FORECAST_CALLS.fetch_add(1, Ordering::SeqCst);
    OpenWeather::forecast(&ForecastArgs { lat, lng, days })
}
