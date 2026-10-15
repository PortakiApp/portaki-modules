//! Weather domain — snapshots, condition mapping, formatting, OpenWeather API.

mod api;
mod condition;
mod format;
mod model;

pub use api::{
    fetch_current_from_api, fetch_forecast_from_api, has_open_weather, map_current, map_forecast,
    CONNECTOR_CURRENT_CALLS, CONNECTOR_FORECAST_CALLS,
};
pub use condition::{description_key_for_condition, icon_name_for_condition, uv_label_key};
pub(crate) use format::parse_forecast_date;
pub use format::{
    city_from_address, convert_temp, format_day_strip_label, format_temp_label, format_wind_kmh,
    resolve_city_label, tone_for_temp_c,
};
pub use model::{
    CachedCurrentPayload, CachedForecastPayload, ForecastDayView, WeatherCurrent, WeatherForecast,
    CURRENT_CACHE_TTL_SECS, FORECAST_CACHE_TTL_SECS,
};
