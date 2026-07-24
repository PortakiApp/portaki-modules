//! Timed password reveal — computed entirely inside the module.
//!
//! ## Fail-safe (missing stay / check-in)
//!
//! - [`RevealPolicy::Always`] → password is revealable even without check-in.
//! - Any other policy → **not revealable** until `checkin_at` is present and
//!   `now >= reveal_at(policy, checkin, timezone)`.
//!
//! ## Timezone for `day_before_16h`
//!
//! Calendar math uses the property IANA timezone. Supported natively:
//! `UTC` / `Etc/UTC` / `GMT`, and common EU zones on CET/CEST
//! (`Europe/Paris`, …). Unknown IANA ids fall back to **UTC** calendar math
//! (still J−1 16:00, but on the UTC date of check-in).

use chrono::{
    DateTime, Datelike, Days, Duration, FixedOffset, NaiveDate, NaiveDateTime, TimeZone, Timelike,
    Utc,
};
use portaki_sdk::prelude::*;

use crate::config::RevealPolicy;

/// Placeholder written into SDUI when the password is not yet revealable.
pub const SECRET_MASK: &str = "••••••";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevealDecision {
    pub revealed: bool,
    pub available_from: Option<DateTime<Utc>>,
}

pub fn evaluate_reveal(
    policy: RevealPolicy,
    now: DateTime<Utc>,
    checkin_at: Option<DateTime<Utc>>,
    property_timezone: &str,
) -> RevealDecision {
    if matches!(policy, RevealPolicy::Always) {
        return RevealDecision {
            revealed: true,
            available_from: None,
        };
    }

    let Some(checkin) = checkin_at else {
        return RevealDecision {
            revealed: false,
            available_from: None,
        };
    };

    let Some(available_from) = reveal_at(policy, checkin, property_timezone) else {
        return RevealDecision {
            revealed: false,
            available_from: None,
        };
    };

    RevealDecision {
        revealed: now >= available_from,
        available_from: Some(available_from),
    }
}

pub fn reveal_at(
    policy: RevealPolicy,
    checkin_at: DateTime<Utc>,
    property_timezone: &str,
) -> Option<DateTime<Utc>> {
    match policy {
        RevealPolicy::Always => None,
        RevealPolicy::HoursBefore24 => Some(checkin_at - Duration::hours(24)),
        RevealPolicy::AtCheckin => Some(checkin_at),
        RevealPolicy::DayBefore16h => day_before_16h(checkin_at, property_timezone),
    }
}

fn day_before_16h(checkin_at: DateTime<Utc>, property_timezone: &str) -> Option<DateTime<Utc>> {
    let offset_at_checkin = offset_for_iana(property_timezone, checkin_at);
    let local_checkin = checkin_at.with_timezone(&offset_at_checkin);
    let target_date = local_checkin.date_naive().checked_sub_days(Days::new(1))?;
    let naive = target_date.and_hms_opt(16, 0, 0)?;
    local_naive_to_utc(property_timezone, naive)
}

fn local_naive_to_utc(tz_name: &str, naive: NaiveDateTime) -> Option<DateTime<Utc>> {
    let utc_guess = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
    let offset = offset_for_iana(tz_name, utc_guess);
    match offset.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => Some(dt.with_timezone(&Utc)),
        chrono::LocalResult::Ambiguous(earliest, _) => Some(earliest.with_timezone(&Utc)),
        chrono::LocalResult::None => {
            let shifted = utc_guess + Duration::hours(1);
            let offset2 = offset_for_iana(tz_name, shifted);
            offset2
                .from_local_datetime(&naive)
                .single()
                .map(|dt| dt.with_timezone(&Utc))
        }
    }
}

fn offset_for_iana(tz_name: &str, at: DateTime<Utc>) -> FixedOffset {
    let name = tz_name.trim();
    if name.is_empty()
        || name.eq_ignore_ascii_case("UTC")
        || name.eq_ignore_ascii_case("Etc/UTC")
        || name.eq_ignore_ascii_case("GMT")
    {
        return FixedOffset::east_opt(0).expect("utc offset");
    }

    if is_europe_cest_zone(name) {
        return europe_cest_offset(at);
    }

    FixedOffset::east_opt(0).expect("utc offset")
}

fn is_europe_cest_zone(name: &str) -> bool {
    matches!(
        name,
        "Europe/Paris"
            | "Europe/Berlin"
            | "Europe/Madrid"
            | "Europe/Rome"
            | "Europe/Brussels"
            | "Europe/Amsterdam"
            | "Europe/Vienna"
            | "Europe/Zurich"
            | "Europe/Luxembourg"
            | "Europe/Monaco"
            | "Europe/Oslo"
            | "Europe/Stockholm"
            | "Europe/Copenhagen"
            | "Europe/Prague"
            | "Europe/Warsaw"
            | "Europe/Budapest"
            | "Europe/Zagreb"
            | "Europe/Ljubljana"
            | "Europe/Bratislava"
            | "Europe/Belgrade"
            | "Europe/Sarajevo"
            | "Europe/Skopje"
            | "Europe/Podgorica"
            | "Europe/Tirane"
            | "Europe/Andorra"
            | "Europe/Malta"
            | "Europe/Vatican"
            | "Europe/San_Marino"
            | "Arctic/Longyearbyen"
    )
}

fn europe_cest_offset(at: DateTime<Utc>) -> FixedOffset {
    let year = at.year();
    let dst_start = last_sunday_of_month(year, 3)
        .and_hms_opt(1, 0, 0)
        .map(|n| Utc.from_utc_datetime(&n))
        .expect("dst start");
    let dst_end = last_sunday_of_month(year, 10)
        .and_hms_opt(1, 0, 0)
        .map(|n| Utc.from_utc_datetime(&n))
        .expect("dst end");
    if at >= dst_start && at < dst_end {
        FixedOffset::east_opt(2 * 3600).expect("cest")
    } else {
        FixedOffset::east_opt(3600).expect("cet")
    }
}

fn last_sunday_of_month(year: i32, month: u32) -> NaiveDate {
    let first_next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .expect("date");
    let last_day = first_next.pred_opt().expect("pred");
    let days_since_sunday = last_day.weekday().num_days_from_sunday();
    last_day
        .checked_sub_days(Days::new(u64::from(days_since_sunday)))
        .expect("sunday")
}

pub fn format_available_from(available_from: DateTime<Utc>, property_timezone: &str) -> String {
    let offset = offset_for_iana(property_timezone, available_from);
    let local = available_from.with_timezone(&offset);
    let day = format!("{:02}", local.day());
    let month = format!("{:02}", local.month());
    let year = local.year().to_string();
    let hour = format!("{:02}", local.hour());
    let minute = format!("{:02}", local.minute());
    let fallback = format!("{day}/{month}/{year} {hour}:{minute}");
    match t!(
        "reveal.availableFrom.datetime",
        day = &day,
        month = &month,
        year = &year,
        hour = &hour,
        minute = &minute
    ) {
        Ok(text) if !looks_like_i18n_key(&text) => text,
        Ok(text) if text.contains('{') => apply_vars(
            &text,
            &[
                ("day", day.as_str()),
                ("month", month.as_str()),
                ("year", year.as_str()),
                ("hour", hour.as_str()),
                ("minute", minute.as_str()),
            ],
        ),
        _ => fallback,
    }
}

pub fn locked_message(available_from_label: Option<&str>, locale: &str) -> String {
    let is_fr = locale.to_ascii_lowercase().starts_with("fr");
    match available_from_label {
        Some(when) => match t!("reveal.locked.withWhen", when = when) {
            Ok(text) if text.contains(when) && !looks_like_i18n_key(&text) => text,
            Ok(text) if text.contains("{when}") => text.replace("{when}", when),
            _ if is_fr => format!("Disponible à partir du {when}"),
            _ => format!("Available from {when}"),
        },
        None => match t!("reveal.locked.generic") {
            Ok(text) if !looks_like_i18n_key(&text) => text,
            _ if is_fr => "Le mot de passe sera disponible à l’approche du check-in.".into(),
            _ => "The password will be available closer to check-in.".into(),
        },
    }
}

fn looks_like_i18n_key(text: &str) -> bool {
    !text.contains(' ') && text.contains('.')
}

fn apply_vars(template: &str, vars: &[(&str, &str)]) -> String {
    let mut text = template.to_string();
    for (name, value) in vars {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s)
            .expect("rfc3339")
            .with_timezone(&Utc)
    }

    #[test]
    fn always_reveals_without_checkin() {
        let d = evaluate_reveal(RevealPolicy::Always, Utc::now(), None, "Europe/Paris");
        assert!(d.revealed);
        assert!(d.available_from.is_none());
    }

    #[test]
    fn missing_checkin_locks_timed_policies() {
        for policy in [
            RevealPolicy::HoursBefore24,
            RevealPolicy::DayBefore16h,
            RevealPolicy::AtCheckin,
        ] {
            let d = evaluate_reveal(policy, Utc::now(), None, "Europe/Paris");
            assert!(!d.revealed, "{policy:?}");
            assert!(d.available_from.is_none());
        }
    }

    #[test]
    fn hours_before_24() {
        let checkin = utc("2026-07-20T14:00:00Z");
        let at = reveal_at(RevealPolicy::HoursBefore24, checkin, "Europe/Paris").unwrap();
        assert_eq!(at, utc("2026-07-19T14:00:00Z"));
    }

    #[test]
    fn day_before_16h_paris_summer() {
        let checkin = utc("2026-07-20T14:00:00Z");
        let at = reveal_at(RevealPolicy::DayBefore16h, checkin, "Europe/Paris").unwrap();
        assert_eq!(at, utc("2026-07-19T14:00:00Z"));
    }
}
