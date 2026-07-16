//! Guest booklet surfaces.

use portaki_sdk::host::{log, module};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{Emphasis, Tone};
use portaki_sdk::sdui::primitives::{
    Badge, Button, Card, Divider, EmptyState, Grid, Icon, InfoBanner, Stack, Text,
};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use crate::config::load_config;
use crate::queries::{get_current, get_forecast, GetCurrentArgs, GetForecastArgs};
use crate::weather::{
    convert_temp, format_day_strip_label, format_temp_label, format_wind_kmh, has_open_weather,
    icon_name_for_condition, is_uv_high, uv_label_key, WeatherCurrent, WeatherForecast,
};

/// Guest home booklet card with current conditions.
#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Surface {
    match render_home_card_inner(&ctx) {
        Ok(surface) => surface,
        Err(error) => {
            log_render_failure("home.card", &error);
            empty_runtime_error_state("home.card")
        }
    }
}

fn render_home_card_inner(ctx: &GuestContext) -> Result<Surface> {
    if let Some(surface) = empty_state_if_module_not_ready("home.card")? {
        return Ok(surface);
    }
    if !has_open_weather(ctx) {
        return Ok(empty_capability_state("home.card"));
    }

    let config = load_config()?;
    let current = get_current(
        ctx.clone(),
        GetCurrentArgs {
            lat: None,
            lng: None,
        },
    )?;
    let forecast = get_forecast(
        ctx.clone(),
        GetForecastArgs {
            lat: None,
            lng: None,
            days: Some(5),
        },
    )?;
    Ok(build_home_card(
        &current,
        &forecast,
        &config.units,
        property_city(ctx),
        &ctx.locale,
    ))
}

fn property_city(ctx: &GuestContext) -> &str {
    let name = ctx.property.name.trim();
    if !name.is_empty() && name != "Property" {
        return name;
    }
    ctx.property
        .address
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(name)
}

/// Same glance body as the card (now + 5-day strip) — shared with the sheet surface.
fn build_weather_body(
    current: &WeatherCurrent,
    forecast: &WeatherForecast,
    units: &crate::entities::WeatherUnits,
    city: &str,
    locale: &str,
) -> Vec<Component> {
    let mut children = vec![build_current_hero(current, units, city)];

    if is_uv_high(current.uv_index) {
        let uv_key = current
            .uv_index
            .map(uv_label_key)
            .unwrap_or("weather.uv.high");
        children.push(Component::Badge(
            Badge::new()
                .label(json!(format!("i18n:{uv_key}")))
                .tone(Tone::Warning),
        ));
    }

    children.push(Component::Divider(Divider::new()));
    children.push(build_forecast_strip(forecast, units, locale));
    children
}

fn build_current_hero(
    current: &WeatherCurrent,
    units: &crate::entities::WeatherUnits,
    city: &str,
) -> Component {
    let temp = convert_temp(current.temp_c, *units);
    let unit = units.sdui_unit();
    let description = json!(format!("i18n:{}", current.description_key));

    Component::Stack(
        Stack::new()
            .direction(json!("horizontal"))
            .gap(json!(16))
            .child(
                Icon::new()
                    .name(json!(icon_name_for_condition(&current.condition)))
                    .size(json!(56)),
            )
            .child(Component::Stack(
                Stack::new()
                    .gap(json!(4))
                    .child(
                        Text::new()
                            .text(json!(format_temp_label(temp, unit, false)))
                            .variant(json!("display")),
                    )
                    .child(Text::new().text(description).variant(json!("caption")))
                    .child(Text::new().text(json!(city)).variant(json!("caption"))),
            )),
    )
}

fn build_home_card(
    current: &WeatherCurrent,
    forecast: &WeatherForecast,
    units: &crate::entities::WeatherUnits,
    city: &str,
    locale: &str,
) -> Surface {
    Surface::new(
        Card::new()
            .icon(json!("cloud-sun"))
            .title(json!("i18n:home.card.title"))
            .action(json!({
                "type": "openOverlay",
                "presentation": "bottomSheet",
                "surfaceRender": "explore.forecast",
                "args": {
                    "icon": "cloud-sun",
                    "title": "i18n:home.card.title"
                }
            }))
            .children(build_weather_body(current, forecast, units, city, locale)),
    )
    .with_id("home.card")
}

fn build_forecast_day_column(
    day: &crate::weather::ForecastDayView,
    unit: &str,
    units: &crate::entities::WeatherUnits,
    locale: &str,
) -> Component {
    let display_temp = convert_temp(day.display_temp_c, *units);
    Component::Stack(
        Stack::new()
            .gap(json!(6))
            .child(
                Text::new()
                    .text(json!(format_day_strip_label(&day.date, locale)))
                    .variant(json!("caption")),
            )
            .child(
                Icon::new()
                    .name(json!(icon_name_for_condition(&day.condition)))
                    .size(json!(28)),
            )
            .child(
                Text::new()
                    .text(json!(format_temp_label(display_temp, unit, false)))
                    .variant(json!("caption"))
                    .emphasis(Emphasis::Strong),
            ),
    )
}

fn build_forecast_strip(
    forecast: &WeatherForecast,
    units: &crate::entities::WeatherUnits,
    locale: &str,
) -> Component {
    let unit = units.sdui_unit();
    let day_columns: Vec<Component> = forecast
        .days
        .iter()
        .map(|day| build_forecast_day_column(day, unit, units, locale))
        .collect();

    Component::Grid(
        Grid::new()
            .columns(json!(5))
            .gap(json!(6))
            .children(day_columns),
    )
}

fn empty_capability_state(surface_id: &str) -> Surface {
    Surface::new(
        EmptyState::new()
            .title(json!("i18n:capability.openWeather.fallback"))
            .description(json!("i18n:capability.openWeather.byok.fallback"))
            .icon(json!("cloud-off"))
            .child(
                Text::new()
                    .text(json!("i18n:home.card.unavailable"))
                    .variant(json!("body")),
            ),
    )
    .with_id(surface_id)
}

fn empty_runtime_error_state(surface_id: &str) -> Surface {
    Surface::new(
        EmptyState::new()
            .title(json!("i18n:home.card.error.title"))
            .description(json!("i18n:home.card.error.description"))
            .icon(json!("cloud-off"))
            .child(
                Text::new()
                    .text(json!("i18n:home.card.unavailable"))
                    .variant(json!("body")),
            ),
    )
    .with_id(surface_id)
}

fn empty_config_state(surface_id: &str) -> Surface {
    Surface::new(
        EmptyState::new()
            .title(json!("i18n:module.status.incomplete.title"))
            .description(json!("i18n:module.status.incomplete.description"))
            .icon(json!("sliders"))
            .child(
                Text::new()
                    .text(json!("i18n:module.status.incomplete.hint"))
                    .variant(json!("body")),
            ),
    )
    .with_id(surface_id)
}

fn empty_inactive_state(surface_id: &str) -> Surface {
    Surface::new(
        EmptyState::new()
            .title(json!("i18n:module.status.inactive.title"))
            .description(json!("i18n:module.status.inactive.description"))
            .icon(json!("cloud-off")),
    )
    .with_id(surface_id)
}

fn log_render_failure(surface_id: &str, error: &PortakiError) {
    let mut fields = log::Fields::new();
    fields.insert("surfaceId", &surface_id);
    fields.insert("error", &error.to_string());
    let _ = log::error("weather_guest_render_failed", &fields);
}

/// Returns an EmptyState when the property-module is not ready to serve guest content.
fn empty_state_if_module_not_ready(surface_id: &str) -> Result<Option<Surface>> {
    let status = module::status()?;
    if !status.workspace_enabled || !status.active {
        return Ok(Some(empty_inactive_state(surface_id)));
    }
    if status.incomplete {
        return Ok(Some(empty_config_state(surface_id)));
    }
    Ok(None)
}

/// Sheet / explore detail — same weather body as the card (design `block("weather")` in sheet).
#[portaki_sdk::surface(guest, id = "explore.forecast")]
pub fn render_explore_forecast(ctx: GuestContext) -> Surface {
    match render_explore_forecast_inner(&ctx) {
        Ok(surface) => surface,
        Err(error) => {
            log_render_failure("explore.forecast", &error);
            empty_runtime_error_state("explore.forecast")
        }
    }
}

fn render_explore_forecast_inner(ctx: &GuestContext) -> Result<Surface> {
    if let Some(surface) = empty_state_if_module_not_ready("explore.forecast")? {
        return Ok(surface);
    }
    if !has_open_weather(ctx) {
        return Ok(empty_capability_state("explore.forecast"));
    }

    let config = load_config()?;
    let current = get_current(
        ctx.clone(),
        GetCurrentArgs {
            lat: None,
            lng: None,
        },
    )?;
    let forecast = get_forecast(
        ctx.clone(),
        GetForecastArgs {
            lat: None,
            lng: None,
            days: Some(5),
        },
    )?;
    Ok(build_sheet_surface(
        &current,
        &forecast,
        &config.units,
        property_city(ctx),
        &ctx.locale,
    ))
}

/// Body-only tree for the bottom sheet (shell supplies header chrome).
fn build_sheet_surface(
    current: &WeatherCurrent,
    forecast: &WeatherForecast,
    units: &crate::entities::WeatherUnits,
    city: &str,
    locale: &str,
) -> Surface {
    let mut children = vec![build_current_hero(current, units, city)];
    if is_uv_high(current.uv_index) {
        let uv_key = current
            .uv_index
            .map(uv_label_key)
            .unwrap_or("weather.uv.high");
        children.push(Component::Badge(
            Badge::new()
                .label(json!(format!("i18n:{uv_key}")))
                .tone(Tone::Warning),
        ));
    }
    children.push(Component::Divider(Divider::new()));
    children.push(build_current_details(current));
    children.push(Component::Divider(Divider::new()));
    children.push(Component::Text(
        Text::new()
            .text(json!("i18n:explore.forecast.hint"))
            .variant(json!("caption")),
    ));
    children.push(build_forecast_table(forecast, units, locale));
    children.push(Component::InfoBanner(
        InfoBanner::new().message(json!("i18n:sheet.assistant.tip")),
    ));
    children.push(Component::Button(
        Button::new().label(json!("i18n:sheet.contactHost")),
    ));

    Surface::new(Stack::new().gap(json!(12)).children(children)).with_id("explore.forecast")
}

fn build_current_details(current: &WeatherCurrent) -> Component {
    let mut cells: Vec<Component> = vec![
        table_header_cell("i18n:weather.humidity"),
        table_value_cell(&format!("{}%", current.humidity)),
    ];

    if let Some(uv) = current.uv_index {
        cells.push(table_header_cell("i18n:weather.uv"));
        cells.push(table_value_cell(&format!("i18n:{}", uv_label_key(uv))));
    }
    if let Some(wind) = current.wind_speed_ms {
        cells.push(table_header_cell("i18n:weather.wind"));
        cells.push(table_value_cell(&format_wind_kmh(wind)));
    }

    Component::Grid(Grid::new().columns(json!(2)).gap(json!(8)).children(cells))
}

/// 4-column forecast table: day · icon · temps · precip.
fn build_forecast_table(
    forecast: &WeatherForecast,
    units: &crate::entities::WeatherUnits,
    locale: &str,
) -> Component {
    let unit = units.sdui_unit();
    let mut cells: Vec<Component> = vec![
        table_header_cell("i18n:weather.col.day"),
        table_header_cell(""),
        table_header_cell("i18n:weather.col.temp"),
        table_header_cell("i18n:weather.col.precip"),
    ];

    for day in &forecast.days {
        let min = format_temp_label(convert_temp(day.min_c, *units), unit, false);
        let max = format_temp_label(convert_temp(day.max_c, *units), unit, false);
        let precip = day
            .precip_chance_pct
            .map(|pct| format!("{pct}%"))
            .unwrap_or_else(|| "—".to_string());

        cells.push(Component::Text(
            Text::new()
                .text(json!(format_day_strip_label(&day.date, locale)))
                .variant(json!("caption"))
                .emphasis(Emphasis::Strong),
        ));
        cells.push(Component::Icon(
            Icon::new()
                .name(json!(icon_name_for_condition(&day.condition)))
                .size(json!(28)),
        ));
        cells.push(Component::Text(
            Text::new()
                .text(json!(format!("{min} / {max}")))
                .variant(json!("caption")),
        ));
        cells.push(Component::Text(
            Text::new().text(json!(precip)).variant(json!("caption")),
        ));
    }

    Component::Grid(Grid::new().columns(json!(4)).gap(json!(10)).children(cells))
}

fn table_header_cell(label: &str) -> Component {
    Component::Text(
        Text::new()
            .text(json!(label))
            .variant(json!("caption"))
            .emphasis(Emphasis::Strong),
    )
}

fn table_value_cell(value: &str) -> Component {
    Component::Text(Text::new().text(json!(value)).variant(json!("caption")))
}
