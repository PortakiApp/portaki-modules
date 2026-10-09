//! Les moyennes mensuelles de la page publique, par OpenWeather Statistical Weather Data.
//!
//! Elles changent peu : chaque mois est gardé 30 jours dans le KV du module, par position arrondie
//! à deux décimales (~1 km, bien moins que le flou de 500 m du centre public). Un échec ne remonte
//! jamais : pas de moyennes, la page montre la température du jour.
//!
//! Deux bornes de la passerelle pèsent ici :
//! - cinq appels de connecteur par invocation, dont deux déjà pris par la météo du jour quand son
//!   cache est vide : [`CALL_BUDGET`] mois au plus par rendu, le reste au rendu suivant ;
//! - 1 500 ms par module sur la page publique : au premier échec (offre sans l'API, le cas le plus
//!   probable), on n'insiste pas, et la position est notée « sans statistiques » un jour.

use chrono::{Datelike, NaiveDate};
use portaki_sdk::host::{connectors, kv, log};
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

pub const CONNECTOR_ID: &str = "open-weather-statistics";
pub const OPERATION: &str = "month";

/// Durée de vie d'une moyenne mensuelle.
pub const MONTH_TTL_SECS: u32 = 30 * 24 * 60 * 60;

/// Après un échec, rien n'est redemandé pour cette position pendant un jour.
const UNAVAILABLE_TTL_SECS: u32 = 24 * 60 * 60;

/// Appels de statistiques par rendu : cinq par invocation, moins les deux de la météo du jour.
pub const CALL_BUDGET: usize = 3;

const KELVIN: f64 = 273.15;

#[derive(Serialize)]
struct MonthArgs {
    lat: f64,
    lon: f64,
    month: u32,
}

#[derive(Deserialize)]
struct MonthResponse {
    result: Option<MonthResult>,
}

#[derive(Deserialize)]
struct MonthResult {
    temp: Option<TempStats>,
}

#[derive(Deserialize)]
struct TempStats {
    mean: Option<f64>,
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// La clé KV d'un mois à une position, arrondie à deux décimales.
pub fn month_key(lat: f64, lng: f64, month: u32) -> String {
    format!("climate.{:.2}.{:.2}.{month}", round2(lat), round2(lng))
}

fn unavailable_key(lat: f64, lng: f64) -> String {
    format!("climate.unavailable.{:.2}.{:.2}", round2(lat), round2(lng))
}

/// Les six mois de la saison : deux avant le mois courant, lui, et trois après.
pub fn season_months(today: NaiveDate) -> [u32; 6] {
    let current = today.month0() as i32;
    std::array::from_fn(|offset| ((current - 2 + offset as i32).rem_euclid(12) + 1) as u32)
}

/// La moyenne en °C de chaque mois, dans l'ordre ; `None` tant qu'il en manque une.
pub fn monthly_means(lat: f64, lng: f64, months: &[u32]) -> Option<Vec<f64>> {
    if matches!(kv::get(&unavailable_key(lat, lng)), Ok(Some(_))) {
        return None;
    }
    let mut budget = CALL_BUDGET;
    let mut means = Vec::with_capacity(months.len());
    for &month in months {
        if let Some(mean) = cached(lat, lng, month) {
            means.push(Some(mean));
            continue;
        }
        if budget == 0 {
            means.push(None);
            continue;
        }
        budget -= 1;
        match fetch(lat, lng, month) {
            Ok(mean) => {
                let _ = kv::set(
                    &month_key(lat, lng, month),
                    mean.to_string().as_bytes(),
                    Some(MONTH_TTL_SECS),
                );
                means.push(Some(mean));
            }
            Err(error) => {
                let mut fields = log::Fields::new();
                fields.insert("month", &month);
                fields.insert("error", &error.to_string());
                let _ = log::warn("weather_climate_unavailable", &fields);
                let _ = kv::set(&unavailable_key(lat, lng), b"1", Some(UNAVAILABLE_TTL_SECS));
                return None;
            }
        }
    }
    means.into_iter().collect()
}

fn cached(lat: f64, lng: f64, month: u32) -> Option<f64> {
    let bytes = kv::get(&month_key(lat, lng, month)).ok()??;
    std::str::from_utf8(&bytes).ok()?.parse().ok()
}

fn fetch(lat: f64, lng: f64, month: u32) -> Result<f64> {
    let args = MonthArgs {
        lat: round2(lat),
        lon: round2(lng),
        month,
    };
    let response: MonthResponse = connectors::call(CONNECTOR_ID, OPERATION, &args)?;
    response
        .result
        .and_then(|result| result.temp)
        .and_then(|temp| temp.mean)
        .map(|kelvin| kelvin - KELVIN)
        .ok_or_else(|| PortakiError::Connector("statistics_without_mean".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(month: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, month, 15).unwrap()
    }

    #[test]
    fn the_season_is_two_months_before_and_three_after() {
        assert_eq!(season_months(day(6)), [4, 5, 6, 7, 8, 9]);
        assert_eq!(season_months(day(1)), [11, 12, 1, 2, 3, 4]);
        assert_eq!(season_months(day(11)), [9, 10, 11, 12, 1, 2]);
    }

    #[test]
    fn the_key_rounds_the_position() {
        assert_eq!(month_key(43.55134, 7.01276, 6), "climate.43.55.7.01.6");
    }
}
