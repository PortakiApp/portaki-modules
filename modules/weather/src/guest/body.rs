//! Shared weather body pieces (hero, 5-day strip).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{Emphasis, TempVariant};
use portaki_sdk::sdui::primitives::{Divider, Grid, Icon, Stack, Temperature, Text};

use crate::entities::WeatherUnits;
use crate::weather::{
    convert_temp, format_day_strip_label, format_temp_label, icon_name_for_condition,
    tone_for_temp_c, ForecastDayView, WeatherCurrent, WeatherForecast,
};

/// Glance body for the home card: hero + forecast strip (design: no UV chip).
pub fn build_weather_glance(
    current: &WeatherCurrent,
    forecast: &WeatherForecast,
    units: &WeatherUnits,
    city: Option<&str>,
) -> Vec<Component> {
    vec![
        build_current_hero(current, units, city),
        Component::Divider(Divider::new()),
        build_forecast_strip(forecast, units),
    ]
}

pub fn build_current_hero(
    current: &WeatherCurrent,
    units: &WeatherUnits,
    city: Option<&str>,
) -> Component {
    let temp = convert_temp(current.temp_c, *units);
    let description = format!("i18n:{}", current.description_key);

    let unit = match units {
        WeatherUnits::Celsius => TemperatureUnit::Celsius,
        WeatherUnits::Fahrenheit => TemperatureUnit::Fahrenheit,
    };
    let mut text_stack = Stack::new().gap(4.0).child(
        Temperature::new()
            .value(temp.round())
            .unit(unit)
            .variant(TempVariant::Hero)
            .tone(tone_for_temp_c(current.temp_c)),
    );
    text_stack = text_stack.child(Text::new().text(description).variant(TextVariant::Caption));
    if let Some(city) = city {
        text_stack = text_stack.child(Text::new().text(city).variant(TextVariant::Caption));
    }

    Component::Stack(
        Stack::new()
            .direction(StackDirection::Horizontal)
            .gap(16.0)
            .child(
                Icon::new()
                    .name(icon_name_for_condition(&current.condition))
                    .size(56.0),
            )
            .child(Component::Stack(text_stack)),
    )
}

/// `today` : le premier jour de la bande. La maquette l'encadre en couleur de marque pour qu'on
/// sache d'où on compte, et met les autres en boîte neutre — sans boîte, les cinq colonnes se
/// lisaient comme un seul bloc de texte.
fn build_forecast_day_column(
    day: &ForecastDayView,
    unit: &str,
    units: &WeatherUnits,
    today: bool,
) -> Component {
    let display_temp = convert_temp(day.display_temp_c, *units);
    let mut column = Stack::new().boxed(true).gap(6.0);
    if today {
        column = column.tone(Tone::Primary);
    }
    Component::Stack(
        column
            .child(
                Text::new()
                    .text(format_day_strip_label(&day.date))
                    .variant(TextVariant::Caption),
            )
            .child(
                Icon::new()
                    .name(icon_name_for_condition(&day.condition))
                    .size(28.0),
            )
            .child(
                Text::new()
                    .text(format_temp_label(display_temp, unit, false))
                    .variant(TextVariant::Caption)
                    .emphasis(Emphasis::Strong)
                    .tone(tone_for_temp_c(day.display_temp_c)),
            ),
    )
}

fn build_forecast_strip(forecast: &WeatherForecast, units: &WeatherUnits) -> Component {
    let unit = units.sdui_unit();
    let day_columns: Vec<Component> = forecast
        .days
        .iter()
        .enumerate()
        .map(|(rank, day)| build_forecast_day_column(day, unit, units, rank == 0))
        .collect();

    // Une largeur minimale, pas un nombre de colonnes : la grille du livret décide combien de
    // jours tiennent sur la largeur qu'elle a (§0.2), et cinq colonnes forcées sur un petit
    // écran donnaient des cases de trois caractères.
    Component::Grid(
        Grid::new()
            .minColumnWidth(44.0)
            .plain(true)
            .gap(6.0)
            .children(day_columns),
    )
}
