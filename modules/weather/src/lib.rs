//! Portaki weather module — current conditions and 5-day forecast via OpenWeather.

mod cache;
mod commands;
mod config;
mod connectors;
mod email_context;
mod entities;
mod events;
mod guest;
mod queries;
mod render_host;
mod weather;

pub use commands::{refresh_forecast, update_config};
pub use email_context::{email_context, EmailContextArgs, EmailContextResponse};
pub use entities::{WeatherCache, WeatherUnits};
pub use events::{on_booking_confirmed, BookingConfirmedEvent};
pub use guest::{render_explore_forecast, render_home_card};
pub use queries::{get_current, get_forecast, GetCurrentArgs, GetForecastArgs};
pub use render_host::render_host_main;
pub use weather::{has_open_weather, WeatherCurrent, WeatherForecast};

pub use cache::{reset_test_cache, reset_test_harness};
pub use weather::{CONNECTOR_CURRENT_CALLS, CONNECTOR_FORECAST_CALLS};

portaki_sdk::portaki_module!(
    id = "weather",
    display_name_key = "module.displayName",
    description_key = "module.description",
    author = "Syntax Labs",
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";

#[portaki_sdk::capability(
    optional,
    id = "external.open-weather.pool",
    purpose_key = "capability.openWeather.purpose",
    fallback_key = "capability.openWeather.fallback"
)]
pub const OPEN_WEATHER_POOL: &str = "external.open-weather.pool";

#[portaki_sdk::capability(
    optional,
    id = "external.open-weather.byok",
    purpose_key = "capability.openWeather.byok.purpose",
    fallback_key = "capability.openWeather.byok.fallback"
)]
pub const OPEN_WEATHER_BYOK: &str = "external.open-weather.byok";
