//! Guest home booklet card.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::Card;
use portaki_sdk::sdui::surface::Surface;

use super::body::build_weather_glance;
use super::load::GuestWeatherData;
use crate::weather::{convert_temp, format_temp_label, icon_name_for_condition};

pub fn build_home_card(data: &GuestWeatherData) -> Surface {
    // Live glance for the nav row — condition icon, current temperature, condition label.
    // The shell renders these generic Card fields; nothing here is shell-specific.
    let icon = icon_name_for_condition(&data.current.condition);
    let temp = format_temp_label(
        convert_temp(data.current.temp_c, data.units),
        data.units.sdui_unit(),
        false,
    );
    let condition = format!("i18n:{}", data.current.description_key);

    Surface::new(
        Card::new()
            .icon(icon)
            .title(temp)
            .subtitle(condition)
            .action(Action::open_overlay(
                OverlayPresentation::BottomSheet,
                crate::ids::EXPLORE_FORECAST,
                OverlayArgs::new().icon(icon).title("i18n:nav.weather"),
            ))
            .children(build_weather_glance(
                &data.current,
                &data.forecast,
                &data.units,
                data.city.as_deref(),
            )),
    )
    .with_id(crate::ids::HOME_CARD)
}
