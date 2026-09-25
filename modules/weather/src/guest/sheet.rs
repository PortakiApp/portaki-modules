//! Guest explore / bottom-sheet forecast surface.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Divider, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use super::body::build_current_hero;
use super::details::build_current_details;
use super::load::GuestWeatherData;
use super::table::build_forecast_table;

/// Body-only tree for the bottom sheet (shell supplies header chrome).
pub fn build_sheet_surface(data: &GuestWeatherData) -> Surface {
    let children = vec![
        build_current_hero(&data.current, &data.units, data.city.as_deref()),
        Component::Divider(Divider::new()),
        build_current_details(&data.current, &data.units),
        Component::Divider(Divider::new()),
        Component::Text(
            Text::new()
                .text("i18n:explore.forecast.hint")
                .variant(TextVariant::Title),
        ),
        build_forecast_table(&data.forecast, &data.units),
    ];

    Surface::new(Stack::new().gap(12.0).children(children)).with_id(crate::guest::EXPLORE_FORECAST)
}
