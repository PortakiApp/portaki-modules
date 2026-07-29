//! Departure label for the guest home card — "Départ {weekday} à {HH:MM}" in the property
//! timezone. The IANA→offset logic mirrors access-guide (EU CEST/CET + UTC fallback); kept
//! module-local until promoted to the SDK.

use chrono::{DateTime, Datelike, Days, FixedOffset, NaiveDate, TimeZone, Timelike, Utc, Weekday};

use portaki_sdk::t;

/// Formats the checkout instant as a localized departure line, or `None` when absent.
pub fn format_departure(checkout_at: Option<DateTime<Utc>>, timezone: &str) -> Option<String> {
    let at = checkout_at?;
    let local = at.with_timezone(&offset_for_iana(timezone, at));
    let weekday_key = weekday_key(local.weekday());
    let weekday = t!(&weekday_key).unwrap_or(weekday_key);
    let time = format!("{:02}:{:02}", local.hour(), local.minute());
    Some(
        t!("guest.departSummary", weekday = &weekday, time = &time)
            .unwrap_or_else(|_| format!("Départ {weekday} · {time}")),
    )
}

fn weekday_key(weekday: Weekday) -> String {
    match weekday {
        Weekday::Mon => "day.monday",
        Weekday::Tue => "day.tuesday",
        Weekday::Wed => "day.wednesday",
        Weekday::Thu => "day.thursday",
        Weekday::Fri => "day.friday",
        Weekday::Sat => "day.saturday",
        Weekday::Sun => "day.sunday",
    }
    .to_string()
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
    // Fail soft: unknown IANA → UTC calendar math.
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
            | "Europe/Lisbon"
    )
}

/// EU DST: last Sunday of March 01:00 UTC → +02:00; last Sunday of October 01:00 UTC → +01:00.
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
