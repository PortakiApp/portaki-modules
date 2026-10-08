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
    let config = ModuleConfig::load(ctx)?;
    if config.point(ctx.property.coordinates).is_none() {
        return Ok(GuestLoad::Empty(Box::new(no_weather(
            surface_id,
            "i18n:guest.noLocation.description",
        ))));
    }

    // La fenêtre du séjour plutôt que cinq jours en dur (§0.7, §2.14) : une semaine à la mer
    // s'arrêtait le mercredi, et avant l'arrivée on lisait surtout des journées qui ne
    // concernaient personne.
    let now = portaki_sdk::host::time::now()?;
    let window = super::window::window(ctx.stay.as_ref(), now, &ctx.timezone);
    let days = super::window::days_to_fetch(window, now, &ctx.timezone);
    // The queries answer `None` when they have no weather to give — the same empty state.
    let (Some(current), Some(forecast)) = (
        get_current(
            ctx.clone(),
            GetCurrentArgs {
                lat: None,
                lng: None,
            },
        )?,
        get_forecast(
            ctx.clone(),
            GetForecastArgs {
                lat: None,
                lng: None,
                days: Some(days),
            },
        )?,
    ) else {
        return Ok(GuestLoad::Empty(Box::new(no_weather(
            surface_id,
            "i18n:guest.unavailable.description",
        ))));
    };
    // Le nom que l'hôte a donné d'abord (« Chamrousse 1750 ») : la commune du fournisseur dit
    // « Chamrousse » pour toute la station.
    let city = config.label().map(str::to_string).or_else(|| {
        resolve_city_label(
            current
                .city_name
                .as_deref()
                .or(forecast.city_name.as_deref()),
            ctx.property.address.as_deref(),
        )
    });

    let forecast = WeatherForecast {
        days: super::window::keep_window(forecast.days, window, |day| day.date.as_str()),
        ..forecast
    };

    Ok(GuestLoad::Ready(Box::new(GuestWeatherData {
        current,
        forecast,
        units: WeatherUnits::for_locale(&ctx.locale),
        city,
    })))
}
