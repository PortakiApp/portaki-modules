//! Current-conditions metric tiles (sheet details).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::Grid;

use crate::entities::WeatherUnits;
use crate::weather::{
    convert_temp, format_temp_label, format_wind_kmh, tone_for_temp_c, uv_label_key, WeatherCurrent,
};

use super::components::metric_tile;

pub fn build_current_details(current: &WeatherCurrent, units: &WeatherUnits) -> Component {
    let unit = units.sdui_unit();
    let mut tiles: Vec<Component> = vec![metric_tile(
        IconName::Droplets,
        "i18n:weather.humidity",
        &format!("{}%", current.humidity),
        None,
    )];

    if let Some(feels) = current.feels_like_c {
        tiles.push(metric_tile(
            IconName::Thermometer,
            "i18n:weather.feelsLike",
            &format_temp_label(convert_temp(feels, *units), unit, false),
            Some(tone_for_temp_c(feels)),
        ));
    }
    if let Some(uv) = current.uv_index {
        tiles.push(metric_tile(
            IconName::Sun,
            "i18n:weather.uv",
            &format!("i18n:{}", uv_label_key(uv)),
            None,
        ));
    }
    if let Some(wind) = current.wind_speed_ms {
        tiles.push(metric_tile(
            IconName::Wind,
            "i18n:weather.wind",
            &format_wind_kmh(wind),
            None,
        ));
    }
    if let Some(pressure) = current.pressure_hpa {
        tiles.push(metric_tile(
            IconName::Gauge,
            "i18n:weather.pressure",
            &format!("{pressure} hPa"),
            None,
        ));
    }
    if let Some(clouds) = current.cloud_pct {
        tiles.push(metric_tile(
            IconName::Cloud,
            "i18n:weather.clouds",
            &format!("{clouds}%"),
            None,
        ));
    }

    Component::Grid(Grid::new().columns(2).gap(12.0).children(tiles))
}
