//! Stay detail encart (spec Wi-Fi §1) : « Révélé le 23/08 à 16:00 » ou « Masqué jusqu'au 23/08 à
//! 16:00 ». Jamais le mot de passe — l'hôte le connaît, l'encart dit seulement quand le voyageur
//! le voit.

use chrono::{DateTime, Datelike, Timelike, Utc};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Page, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{ModuleConfig, RevealPolicy};
use crate::reveal::{evaluate_reveal, offset_for_iana};

#[portaki_sdk::surface(
    host,
    id = "stay",
    placement = HostPlacement::StayDetail,
    label_key = "catalog.host.stay",
    icon = IconName::Wifi
)]
pub fn render_host_stay(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    let line = if config.is_empty() {
        "i18n:host.stay.noNetwork".to_string()
    } else {
        match window(&ctx) {
            Some((checkin, checkout)) => line(
                &config,
                checkin,
                checkout,
                time::now()?,
                &ctx.property.timezone,
            ),
            // Pas de fenêtre (séjour sans dates, ou plateforme plus ancienne) : un tiret et une
            // phrase plutôt qu'une date inventée.
            None => "i18n:host.stay.unknown".to_string(),
        }
    };
    let card = Card::new()
        .icon(IconName::Wifi)
        .title("i18n:host.stay.title")
        .child(Text::new().text(line).variant(TextVariant::Body));
    Ok(Surface::new(Page::new().child(card)).with_id(STAY))
}

/// `input.stay` : la fenêtre que la plateforme joint à une surface de fiche séjour.
fn window(ctx: &HostContext) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let stay = ctx.input.get("stay")?;
    let parse = |key: &str| {
        stay.get(key)
            .and_then(|v| v.as_str())
            .and_then(|v| DateTime::parse_from_rfc3339(v).ok())
            .map(|at| at.with_timezone(&Utc))
    };
    Some((parse("checkIn")?, parse("checkOut")?))
}

fn line(
    config: &ModuleConfig,
    checkin: DateTime<Utc>,
    checkout: DateTime<Utc>,
    now: DateTime<Utc>,
    timezone: &str,
) -> String {
    if config.reveal_policy == RevealPolicy::Always {
        return "i18n:host.stay.always".to_string();
    }
    let decision = evaluate_reveal(
        config.reveal_policy,
        now,
        Some(checkin),
        Some(checkout),
        timezone,
    );
    if decision.ended {
        return "i18n:host.stay.ended".to_string();
    }
    let Some(at) = decision.available_from else {
        return "i18n:host.stay.unknown".to_string();
    };
    let local = at.with_timezone(&offset_for_iana(timezone, at));
    let (day, month) = (
        format!("{:02}", local.day()),
        format!("{:02}", local.month()),
    );
    let (hour, minute) = (
        format!("{:02}", local.hour()),
        format!("{:02}", local.minute()),
    );
    let key = if decision.revealed {
        "host.stay.revealed"
    } else {
        "host.stay.hidden"
    };
    let fallback = if decision.revealed {
        format!("Révélé le {day}/{month} à {hour}:{minute}")
    } else {
        format!("Masqué jusqu'au {day}/{month} à {hour}:{minute}")
    };
    match t!(
        key,
        day = &day,
        month = &month,
        hour = &hour,
        minute = &minute
    ) {
        Ok(text) if text.contains(&day) => text,
        _ => fallback,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn the_day_before_at_four_pm_in_the_property_time() {
        let config = ModuleConfig::default(); // la veille à 16 h
        let (checkin, checkout) = (at("2026-08-24T14:00:00Z"), at("2026-08-29T08:00:00Z"));
        assert_eq!(
            line(
                &config,
                checkin,
                checkout,
                at("2026-08-20T10:00:00Z"),
                "Europe/Paris"
            ),
            "Masqué jusqu'au 23/08 à 16:00"
        );
        assert_eq!(
            line(
                &config,
                checkin,
                checkout,
                at("2026-08-23T15:00:00Z"),
                "Europe/Paris"
            ),
            "Révélé le 23/08 à 16:00"
        );
        assert_eq!(
            line(
                &config,
                checkin,
                checkout,
                at("2026-08-30T10:00:00Z"),
                "Europe/Paris"
            ),
            "i18n:host.stay.ended"
        );
    }
}
