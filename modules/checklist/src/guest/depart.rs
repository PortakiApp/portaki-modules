//! Departure label for the guest home card — "Départ {weekday} à {HH:MM}" in the property
//! timezone (offset from `domain::timezone`).

use chrono::{DateTime, FixedOffset, Timelike, Utc, Weekday};

use portaki_sdk::host::time::{relative_day, short_date, RelativeDay};
use portaki_sdk::t;

use crate::domain::timezone::offset_for_iana;

/// Formats the checkout instant as a localized departure line, or `None` when absent.
///
/// The day reads from `now`: « Départ demain à 11:00 » the eve, « aujourd'hui » on the day,
/// « hier » after, the weekday within the week, else the date — a weekday alone would name next
/// week's Monday too.
pub fn format_departure(
    checkout_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
    timezone: &str,
    locale: &str,
) -> Option<String> {
    let at = checkout_at?;
    let local = at.with_timezone(&offset_for_iana(timezone, at));
    let today = now.with_timezone(&offset_for_iana(timezone, now));
    let key = match relative_day(local.naive_local(), today.naive_local()) {
        RelativeDay::Today | RelativeDay::Tonight => "day.today".to_string(),
        RelativeDay::Tomorrow => "day.tomorrow".to_string(),
        RelativeDay::Yesterday => "day.yesterday".to_string(),
        RelativeDay::Weekday(day) => weekday_key(day),
        // `Date`, et tout cas que le SDK ajouterait : la date.
        _ => return Some(summary(&short_date(local.date_naive(), locale), local)),
    };
    Some(summary(&t!(&key).unwrap_or(key), local))
}

/// « Départ {weekday} à {time} », `weekday` being the day as the guest reads it.
fn summary(weekday: &str, local: DateTime<FixedOffset>) -> String {
    let time = format!("{:02}:{:02}", local.hour(), local.minute());
    t!("guest.departSummary", weekday = weekday, time = &time)
        .unwrap_or_else(|_| format!("Départ {weekday} · {time}"))
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
