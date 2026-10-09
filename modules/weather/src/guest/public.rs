//! Bloc Climat de la page publique du logement (`property.public`, tranche 5).
//!
//! Un visiteur sans séjour : la surface ne lit jamais `ctx.stay`, et la position est le centre
//! flouté que la plateforme donne, assez précis pour la météo. La température du jour, puis les
//! moyennes des six mois de la saison quand l'hôte les veut et qu'OpenWeather les donne. Bloc
//! coupé, ou pas même la météo du jour : une `Section` sans enfant, que la plateforme masque.

use portaki_sdk::host::{log, time};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{Emphasis, GeoPoint};
use portaki_sdk::sdui::primitives::{Grid, Section, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::climate::{monthly_means, season_months};
use crate::config::ModuleConfig;
use crate::entities::WeatherUnits;
use crate::queries::{get_current, GetCurrentArgs};
use crate::weather::{convert_temp, format_temp_label, has_open_weather, tone_for_temp_c};

use super::body::build_current_hero;

const MONTH_KEYS: [&str; 12] = [
    "i18n:month.jan",
    "i18n:month.feb",
    "i18n:month.mar",
    "i18n:month.apr",
    "i18n:month.may",
    "i18n:month.jun",
    "i18n:month.jul",
    "i18n:month.aug",
    "i18n:month.sep",
    "i18n:month.oct",
    "i18n:month.nov",
    "i18n:month.dec",
];

/// Le bloc Climat de la page publique.
#[portaki_sdk::surface(guest, id = "property.public")]
pub fn render_property_public(ctx: GuestContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    let children = if config.public_enabled {
        content(&ctx, &config)
    } else {
        Vec::new()
    };
    let section = Section::new()
        .title("i18n:public.title")
        .subtitle("i18n:public.eyebrow")
        .children(children);
    Ok(Surface::new(section).with_id(ids::convention::PROPERTY_PUBLIC))
}

fn content(ctx: &GuestContext, config: &ModuleConfig) -> Vec<Component> {
    if !has_open_weather(ctx) {
        return Vec::new();
    }
    let Some(point) = config.point(ctx.property.coordinates) else {
        return Vec::new();
    };
    let current = match get_current(
        ctx.clone(),
        GetCurrentArgs {
            lat: None,
            lng: None,
        },
    ) {
        Ok(Some(current)) => current,
        Ok(None) => return Vec::new(),
        Err(error) => {
            let mut fields = log::Fields::new();
            fields.insert("error", &error.to_string());
            let _ = log::warn("weather_public_current_failed", &fields);
            return Vec::new();
        }
    };
    let units = WeatherUnits::for_locale(&ctx.locale);
    let mut children = vec![build_current_hero(&current, &units, None)];
    if config.public_monthly_averages {
        if let Some(grid) = season_grid(ctx, point, units) {
            children.push(
                Text::new()
                    .text("i18n:public.averages")
                    .variant(TextVariant::Caption)
                    .into(),
            );
            children.push(grid);
        }
    }
    children
}

/// Les six mois de la saison, ou rien tant qu'une moyenne manque.
fn season_grid(ctx: &GuestContext, point: GeoPoint, units: WeatherUnits) -> Option<Component> {
    let now = time::now().ok()?;
    let today = match time::PropertyTz::parse(&ctx.timezone) {
        Some(tz) => tz.to_local(now).date_naive(),
        None => now.date_naive(),
    };
    let months = season_months(today);
    let means = monthly_means(point.lat, point.lng, &months)?;
    let current = months[2];
    let columns = months
        .iter()
        .zip(means)
        .map(|(&month, mean_c)| month_column(month, mean_c, units, month == current))
        .collect();
    Some(Component::Grid(
        Grid::new()
            .minColumnWidth(44.0)
            .plain(true)
            .gap(6.0)
            .children(columns),
    ))
}

fn month_column(month: u32, mean_c: f64, units: WeatherUnits, current: bool) -> Component {
    let mut column = Stack::new().boxed(true).gap(4.0);
    if current {
        column = column.tone(Tone::Primary);
    }
    Component::Stack(
        column
            .child(
                Text::new()
                    .text(month_label_key(month))
                    .variant(TextVariant::Caption),
            )
            .child(
                Text::new()
                    .text(format_temp_label(
                        convert_temp(mean_c, units),
                        units.sdui_unit(),
                        false,
                    ))
                    .variant(TextVariant::Caption)
                    .emphasis(Emphasis::Strong)
                    .tone(tone_for_temp_c(mean_c)),
            ),
    )
}

/// `i18n:month.jan` … `i18n:month.dec` pour un mois de 1 à 12.
pub fn month_label_key(month: u32) -> &'static str {
    MONTH_KEYS[(month as usize + 11) % 12]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_month_has_its_short_label() {
        assert_eq!(month_label_key(1), "i18n:month.jan");
        assert_eq!(month_label_key(12), "i18n:month.dec");
    }
}
