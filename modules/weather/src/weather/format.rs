//! Display helpers for guest / host surfaces.

use chrono::{Datelike, NaiveDate};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;

use crate::entities::WeatherUnits;

/// Converts Celsius to the configured display unit.
pub fn convert_temp(value_c: f64, units: WeatherUnits) -> f64 {
    match units {
        WeatherUnits::Celsius => value_c,
        WeatherUnits::Fahrenheit => (value_c * 9.0 / 5.0) + 32.0,
    }
}

/// Formats a temperature for `Text` (hero omits unit letter — design shows `24°`).
pub fn format_temp_label(temp: f64, unit: &str, include_unit: bool) -> String {
    let rounded = temp.round() as i64;
    if !include_unit {
        return format!("{rounded}°");
    }
    let letter = match unit {
        "fahrenheit" | "F" | "f" => "F",
        _ => "C",
    };
    format!("{rounded}°{letter}")
}

/// Wind speed label in km/h for guest display.
pub fn format_wind_kmh(wind_speed_ms: f64) -> String {
    let kmh = (wind_speed_ms * 3.6).round() as i64;
    format!("{kmh} km/h")
}

/// Tone for a Celsius temperature (cool → warm).
pub fn tone_for_temp_c(temp_c: f64) -> Tone {
    if temp_c < 12.0 {
        Tone::Info
    } else if temp_c < 22.0 {
        Tone::Success
    } else if temp_c < 28.0 {
        Tone::Warning
    } else {
        Tone::Danger
    }
}

/// Short strip / table label: « Aujourd'hui », « Demain », then weekday + day-of-month.
///
/// Les deux premiers jours se disent relativement : c'est ainsi qu'on les lit, et la démo du
/// livret, rendue à une date figée, ne tombe juste sur un nom de jour que par hasard.
pub fn format_day_strip_label(date: &str, today: NaiveDate) -> String {
    use portaki_sdk::host::time::{relative_day, RelativeDay};

    let Some(parsed) = parse_forecast_date(date) else {
        return date.to_string();
    };
    let relative = match relative_day(parsed.into(), today.into()) {
        RelativeDay::Today | RelativeDay::Tonight => Some("day.today"),
        RelativeDay::Tomorrow => Some("day.tomorrow"),
        _ => None,
    };
    if let Some(key) = relative {
        return t!(key).unwrap_or_else(|_| key.to_string());
    }
    let day = parsed.day().to_string();
    let key = weekday_key_for_date(date);
    let weekday = t!(&key).unwrap_or(key);
    t!("day.strip", weekday = &weekday, day = &day).unwrap_or_else(|_| format!("{weekday} {day}"))
}

/// Resolves a locality label — prefers OpenWeather city, then address locality.
/// Never falls back to the property marketing name (e.g. "Vayoux").
pub fn resolve_city_label(api_city: Option<&str>, address: Option<&str>) -> Option<String> {
    if let Some(city) = api_city.map(str::trim).filter(|value| !value.is_empty()) {
        return Some(city.to_string());
    }
    address
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(city_from_address)
        .map(str::to_string)
}

pub(crate) fn weekday_key_for_date(date: &str) -> String {
    let Some(parsed) = parse_forecast_date(date) else {
        return "day.monday".to_string();
    };
    match parsed.weekday().number_from_monday() {
        1 => "day.monday",
        2 => "day.tuesday",
        3 => "day.wednesday",
        4 => "day.thursday",
        5 => "day.friday",
        6 => "day.saturday",
        _ => "day.sunday",
    }
    .to_string()
}

pub(crate) fn parse_forecast_date(date: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
}

/// La commune d'une adresse d'une ligne. D'abord le segment « 45000 Orléans » (code postal à cinq
/// chiffres, la règle `POSTAL_CITY` du dashboard) ; sinon le segment avant un pays final
/// (« Cannes, France »), sinon le dernier segment, code postal retiré.
pub fn city_from_address(address: &str) -> Option<&str> {
    let segments: Vec<&str> = address
        .split(',')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect();
    let postal = segments.iter().find_map(|segment| {
        let (code, rest) = segment.split_once(char::is_whitespace)?;
        let rest = rest.trim();
        (code.len() == 5 && code.bytes().all(|b| b.is_ascii_digit()) && !rest.is_empty())
            .then_some(rest)
    });
    if postal.is_some() {
        return postal;
    }
    let last = match segments.as_slice() {
        [.., city, country] if is_country(country) => city,
        [.., last] => last,
        [] => return None,
    };
    Some(last.trim_start_matches(|c: char| c.is_ascii_digit() || c == '-' || c == ' '))
        .filter(|city| !city.is_empty())
}

// ponytail: les pays des logements servis aujourd'hui ; un géocodeur qui rend la commune à part
// remplacerait cette liste.
fn is_country(segment: &str) -> bool {
    const COUNTRIES: &[&str] = &[
        "france",
        "belgique",
        "belgië",
        "belgium",
        "suisse",
        "schweiz",
        "switzerland",
        "luxembourg",
        "monaco",
        "italia",
        "italy",
        "italie",
        "españa",
        "spain",
        "espagne",
        "portugal",
        "deutschland",
        "germany",
        "allemagne",
        "nederland",
        "netherlands",
        "pays-bas",
        "united kingdom",
        "royaume-uni",
        "uk",
        "united states",
        "usa",
        "états-unis",
        "canada",
        "maroc",
        "morocco",
    ];
    COUNTRIES.contains(&segment.to_lowercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_sdk::sdui::common::Tone;
    use portaki_test_utils::MockContext;

    #[test]
    fn formats_strip_label_via_i18n() {
        let today = NaiveDate::from_ymd_opt(2026, 7, 14).unwrap();
        MockContext::guest()
            .with_translation("day.today", "Aujourd'hui")
            .with_translation("day.tomorrow", "Demain")
            .with_translation("day.thursday", "jeu.")
            .with_translation("day.strip", "{weekday} {day}")
            .run(|_| {
                assert_eq!(format_day_strip_label("2026-07-14", today), "Aujourd'hui");
                assert_eq!(format_day_strip_label("2026-07-15", today), "Demain");
                assert_eq!(format_day_strip_label("2026-07-16", today), "jeu. 16");
            });
    }

    #[test]
    fn resolve_city_prefers_api_over_address() {
        assert_eq!(
            resolve_city_label(Some("Cannes"), Some("12 rue X, 06400 Vayoux")),
            Some("Cannes".to_string())
        );
    }

    #[test]
    fn resolve_city_from_address_locality() {
        assert_eq!(
            resolve_city_label(None, Some("12 rue des Lilas, 06400 Antibes")),
            Some("Antibes".to_string())
        );
    }

    #[test]
    fn the_commune_is_read_not_the_country() {
        assert_eq!(
            city_from_address("3 quai X, 45000 Orléans, France"),
            Some("Orléans")
        );
        assert_eq!(
            city_from_address("12 rue du Port, 56000 Vannes"),
            Some("Vannes")
        );
        assert_eq!(
            city_from_address("Chemin des Crêtes, Cannes, France"),
            Some("Cannes")
        );
        assert_eq!(city_from_address("Rua Augusta, Lisboa"), Some("Lisboa"));
    }

    #[test]
    fn resolve_city_skips_empty() {
        assert_eq!(resolve_city_label(Some("  "), None), None);
        assert_eq!(resolve_city_label(None, None), None);
    }

    #[test]
    fn tone_for_temp_bands() {
        assert_eq!(tone_for_temp_c(8.0), Tone::Info);
        assert_eq!(tone_for_temp_c(18.0), Tone::Success);
        assert_eq!(tone_for_temp_c(25.0), Tone::Warning);
        assert_eq!(tone_for_temp_c(32.0), Tone::Danger);
    }
}
