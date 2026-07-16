//! Guest home booklet card.

use portaki_sdk::sdui::primitives::Card;
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use super::body::build_weather_glance;
use super::load::GuestWeatherData;

pub fn build_home_card(data: &GuestWeatherData) -> Surface {
    Surface::new(
        Card::new()
            .icon(json!("cloud-sun"))
            .title(json!("i18n:home.card.title"))
            .action(json!({
                "type": "navigate",
                "to": "weather/forecast"
            }))
            .children(build_weather_glance(
                &data.current,
                &data.forecast,
                &data.units,
                data.city.as_deref(),
                &data.locale,
            )),
    )
    .with_id("home.card")
}
