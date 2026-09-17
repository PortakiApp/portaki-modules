//! Guest-email weather summary for Portaki `arrival-day` (and future templates).

use portaki_sdk::prelude::*;

use crate::queries::{get_current, GetCurrentArgs};
use crate::weather::{resolve_city_label, WeatherCurrent};

/// Arguments for `emailContext`.
#[portaki_sdk::wire]
#[derive(Default)]
pub struct EmailContextArgs {
    /// Portaki template key (`arrival-day`, …).
    #[serde(default)]
    pub template_key: Option<EmailTemplateKey>,
    /// Optional address hint when OpenWeather city is empty.
    #[serde(default)]
    pub address_hint: Option<String>,
    /// Optional locale override (BCP-47). Kept for wire compat; copy uses host i18n.
    #[serde(default)]
    pub locale: Option<String>,
}

/// Email-ready weather contribution.
#[portaki_sdk::wire]
#[derive(PartialEq, Eq)]
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
    if let Some(key) = args.template_key {
        if key != EmailTemplateKey::ArrivalDay {
            return Ok(EmailContextResponse {
                weather_summary: None,
            });
        }
    }

    let current = get_current(
        ctx.clone(),
        GetCurrentArgs {
            lat: None,
            lng: None,
        },
    )?;

    let address = args
        .address_hint
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or(ctx.property.address.as_deref());

    Ok(EmailContextResponse {
        weather_summary: Some(format_weather_summary(&current, address)?),
    })
}

fn format_weather_summary(current: &WeatherCurrent, address: Option<&str>) -> Result<String> {
    let city = resolve_city_label(current.city_name.as_deref(), address);
    let place = match city.as_deref() {
        Some(name) if !name.is_empty() => t!("email.place.inCity", name = name)?,
        _ => t!("email.place.onSite")?,
    };
    let rounded = current.temp_c.round() as i64;
    let condition = condition_phrase(&current.description_key, &current.condition)?;
    let emoji = condition_emoji(&current.condition);

    t!(
        "email.weather.summary",
        place = place,
        emoji = emoji,
        temp = rounded,
        condition = condition
    )
}

fn condition_phrase(description_key: &str, condition: &str) -> Result<String> {
    let key = description_key.trim();
    if key.ends_with("sunny") || key.contains(".sunny") {
        return t!("email.condition.sunny");
    }
    if key.ends_with("cloudy") || key.contains(".cloudy") {
        return t!("email.condition.cloudy");
    }
    if key.ends_with("rainy") || key.contains(".rainy") {
        return t!("email.condition.rainy");
    }
    if key.ends_with("snowy") || key.contains(".snowy") {
        return t!("email.condition.snowy");
    }
    if key.ends_with("stormy") || key.contains(".stormy") {
        return t!("email.condition.stormy");
    }
    if key.ends_with("foggy") || key.contains(".foggy") {
        return t!("email.condition.foggy");
    }

    let c = condition.trim().to_ascii_lowercase();
    if c.contains("clear") || c.contains("sun") {
        return t!("email.condition.sunny");
    }
    if c.contains("cloud") {
        return t!("email.condition.cloudy");
    }
    if c.contains("rain") || c.contains("drizzle") {
        return t!("email.condition.rainy");
    }
    t!("email.condition.variable")
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

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // tests natifs : l'horloge du système y est disponible
mod tests {
    use super::*;
    use crate::entities::WeatherUnits;
    use chrono::Utc;
    use portaki_test_utils::MockContext;

    #[test]
    fn formats_summary_via_i18n() {
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

        MockContext::guest()
            .with_translation("email.place.inCity", "à {name}")
            .with_translation(
                "email.weather.summary",
                "Météo {place} aujourd'hui : {emoji} {temp}°C, {condition}.",
            )
            .with_translation("email.condition.sunny", "ciel dégagé")
            .run(|_| {
                let summary = format_weather_summary(&current, None).expect("summary");
                assert!(summary.contains("Antibes"));
                assert!(summary.contains("27°C"));
                assert!(summary.contains("aujourd'hui"));
            });
    }
}
