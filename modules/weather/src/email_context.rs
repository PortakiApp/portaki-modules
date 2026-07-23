//! Guest-email weather summary for Portaki `arrival-day` (and future templates).

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::queries::{get_current, GetCurrentArgs};
use crate::weather::{resolve_city_label, WeatherCurrent};

/// Arguments for `emailContext`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EmailContextArgs {
    /// Portaki template key (`arrival-day`, …).
    #[serde(default)]
    pub template_key: Option<String>,
    /// Optional address hint when OpenWeather city is empty.
    #[serde(default)]
    pub address_hint: Option<String>,
    /// Optional locale override (BCP-47). Falls back to `ctx.locale`.
    #[serde(default)]
    pub locale: Option<String>,
}

/// Email-ready weather contribution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EmailContextResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weather_summary: Option<String>,
}

/// Email-ready weather sentence for Portaki guest templates.
#[portaki_sdk::query(name = "emailContext")]
pub fn email_context(ctx: Context, args: EmailContextArgs) -> Result<EmailContextResponse> {
    build_email_context(ctx, args)
}

/// Resolve current conditions into a single email sentence.
pub fn build_email_context(ctx: Context, args: EmailContextArgs) -> Result<EmailContextResponse> {
    let template = args.template_key.as_deref().unwrap_or("").trim();
    if !template.is_empty() && template != "arrival-day" {
        return Ok(EmailContextResponse {
            weather_summary: None,
        });
    }

    let current = get_current(
        ctx.clone(),
        GetCurrentArgs {
            lat: None,
            lng: None,
        },
    )?;

    let locale = args
        .locale
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(ctx.locale.as_str());
    let address = args
        .address_hint
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or(ctx.property.address.as_deref());

    Ok(EmailContextResponse {
        weather_summary: Some(format_weather_summary(&current, address, locale)),
    })
}

fn format_weather_summary(current: &WeatherCurrent, address: Option<&str>, locale: &str) -> String {
    let city = resolve_city_label(current.city_name.as_deref(), address);
    let place = match city.as_deref() {
        Some(name) if !name.is_empty() => {
            if locale_is_en(locale) {
                format!("in {name}")
            } else {
                format!("à {name}")
            }
        }
        _ => {
            if locale_is_en(locale) {
                "on site".into()
            } else {
                "sur place".into()
            }
        }
    };
    let rounded = current.temp_c.round() as i64;
    let condition_fr = condition_phrase(&current.description_key, &current.condition, locale);
    let emoji = condition_emoji(&current.condition);

    if locale_is_en(locale) {
        format!("Weather {place} today: {emoji} {rounded}°C, {condition_fr}.")
    } else {
        format!("Météo {place} aujourd'hui : {emoji} {rounded}°C, {condition_fr}.")
    }
}

fn condition_phrase(description_key: &str, condition: &str, locale: &str) -> String {
    let key = description_key.trim();
    let en = locale_is_en(locale);
    if key.ends_with("sunny") || key.contains(".sunny") {
        return if en {
            "clear skies".into()
        } else {
            "ciel dégagé".into()
        };
    }
    if key.ends_with("cloudy") || key.contains(".cloudy") {
        return if en {
            "cloudy".into()
        } else {
            "ciel nuageux".into()
        };
    }
    if key.ends_with("rainy") || key.contains(".rainy") {
        return if en {
            "rain possible".into()
        } else {
            "pluie possible".into()
        };
    }
    if key.ends_with("snowy") || key.contains(".snowy") {
        return if en {
            "snow possible".into()
        } else {
            "neige possible".into()
        };
    }
    if key.ends_with("stormy") || key.contains(".stormy") {
        return if en {
            "storms possible".into()
        } else {
            "orages possibles".into()
        };
    }
    if key.ends_with("foggy") || key.contains(".foggy") {
        return if en { "fog".into() } else { "brume".into() };
    }

    let c = condition.trim().to_ascii_lowercase();
    if c.contains("clear") || c.contains("sun") {
        return if en {
            "clear skies".into()
        } else {
            "ciel dégagé".into()
        };
    }
    if c.contains("cloud") {
        return if en {
            "cloudy".into()
        } else {
            "ciel nuageux".into()
        };
    }
    if c.contains("rain") || c.contains("drizzle") {
        return if en {
            "rain possible".into()
        } else {
            "pluie possible".into()
        };
    }
    if en {
        "variable conditions".into()
    } else {
        "conditions variables".into()
    }
}

fn condition_emoji(condition: &str) -> &'static str {
    let c = condition.trim().to_ascii_lowercase();
    if c.contains("clear") || c.contains("sun") {
        "☀️"
    } else if c.contains("rain") || c.contains("drizzle") {
        "🌧️"
    } else if c.contains("snow") {
        "❄️"
    } else if c.contains("storm") || c.contains("thunder") {
        "⛈️"
    } else if c.contains("cloud") {
        "☁️"
    } else {
        "🌤️"
    }
}

fn locale_is_en(locale: &str) -> bool {
    locale.to_ascii_lowercase().starts_with("en")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::WeatherUnits;
    use chrono::Utc;

    #[test]
    fn formats_french_summary_with_city() {
        let current = WeatherCurrent {
            temp_c: 27.4,
            condition: "Clear".into(),
            humidity: 50,
            uv_index: None,
            wind_speed_ms: None,
            city_name: Some("Antibes".into()),
            feels_like_c: None,
            pressure_hpa: None,
            cloud_pct: None,
            description_key: "weather.description.sunny".into(),
            units: WeatherUnits::Celsius,
            fetched_at: Utc::now(),
        };
        let summary = format_weather_summary(&current, None, "fr");
        assert!(summary.contains("Antibes"));
        assert!(summary.contains("27°C"));
        assert!(summary.contains("ciel dégagé"));
        assert!(summary.contains("☀️"));
    }
}
