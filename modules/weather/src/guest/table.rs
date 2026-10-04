//! Les prévisions, jour par jour, pour la feuille d'exploration (§2.14).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{Leading, LeadingVisual, SurfaceLevel, Trailing, TrailingVisual};
use portaki_sdk::sdui::primitives::{Card, ListItem};

use crate::entities::WeatherUnits;
use crate::weather::{
    convert_temp, description_key_for_condition, format_day_strip_label, format_temp_label,
    format_wind_kmh, icon_name_for_condition, WeatherForecast,
};

/// Une ligne par jour : l'icône du temps devant, le jour, ce qu'il fait, et les deux températures
/// en fin de ligne.
///
/// Remplace la grille à sept colonnes : sur un téléphone, sept colonnes se serrent jusqu'à ce que
/// chacune tienne trois caractères, et un en-tête « 💧 » au-dessus d'un « 70 % » n'a plus de mot
/// pour dire ce que c'est. Le détail — vent, pluie, humidité — passe en sous-titre, où il se lit
/// en une phrase.
pub fn build_forecast_table(forecast: &WeatherForecast, units: &WeatherUnits) -> Component {
    let unit = units.sdui_unit();
    let rows: Vec<Component> = forecast
        .days
        .iter()
        .enumerate()
        .map(|(rank, day)| {
            let min = format_temp_label(convert_temp(day.min_c, *units), unit, false);
            let max = format_temp_label(convert_temp(day.max_c, *units), unit, false);

            let mut item = ListItem::new()
                .title(format_day_strip_label(&day.date))
                .subtitle(day_summary(day))
                // Les deux températures dans l'emplacement de fin, en chiffres à chasse fixe :
                // une colonne de « 18° / 22° » ne s'aligne pas en proportionnelle.
                .trailing(Trailing::Visual(Box::new(TrailingVisual {
                    text: Some(format!("{min} / {max}")),
                    mono: true,
                    ..TrailingVisual::default()
                })))
                .leading(Leading::Visual(Box::new(LeadingVisual {
                    icon: Some(icon_name_for_condition(&day.condition)),
                    ..LeadingVisual::default()
                })));
            // Le premier jour est aujourd'hui : la maquette le teinte pour qu'on le retrouve
            // sans compter les lignes.
            if rank == 0 {
                item = item.tone(Tone::Primary);
            }
            Component::ListItem(item)
        })
        .collect();

    Component::Card(
        Card::new()
            .surface(SurfaceLevel::Elevated)
            .title("i18n:explore.forecast.days")
            .children(rows),
    )
}

/// « Ensoleillé · vent 25 km/h · pluie 70 % » — ce que la journée a de notable, sans trou.
fn day_summary(day: &crate::weather::ForecastDayView) -> String {
    let mut parts: Vec<String> = Vec::new();
    let condition = t!(description_key_for_condition(&day.condition).as_str()).unwrap_or_default();
    if !condition.is_empty() {
        parts.push(condition);
    }
    if let Some(wind) = day.wind_speed_ms_max.map(format_wind_kmh) {
        if let Ok(line) = t!("weather.summary.wind", value = wind) {
            parts.push(line);
        }
    }
    if let Some(precip) = day.precip_chance_pct {
        if let Ok(line) = t!("weather.summary.precip", value = precip) {
            parts.push(line);
        }
    }
    if let Some(humidity) = day.humidity_avg {
        if let Ok(line) = t!("weather.summary.humidity", value = humidity) {
            parts.push(line);
        }
    }
    parts.join(" · ")
}
