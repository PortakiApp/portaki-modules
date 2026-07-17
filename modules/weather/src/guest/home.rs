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
            .title(json!("i18n:nav.weather"))
            .action(json!({
                "type": "openOverlay",
                "presentation": "bottomSheet",
                "surfaceRender": "explore.forecast",
                "args": {
                    "icon": "cloud-sun",
                    "title": "i18n:nav.weather"
                }
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
