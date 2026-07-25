//! Guest checklist availability from `ModuleConfig.show_when` + stay window.

use chrono::{DateTime, Duration, TimeZone, Utc};

use crate::config::ShowWhen;

/// Whether the guest checklist should be shown right now.
pub fn is_checklist_available(
    policy: ShowWhen,
    now: DateTime<Utc>,
    checkin_at: Option<DateTime<Utc>>,
    checkout_at: Option<DateTime<Utc>>,
) -> bool {
    match policy {
        ShowWhen::Always => true,
        ShowWhen::FromCheckin => match checkin_at {
            // No check-in yet — fail open so the checklist stays reachable.
            None => true,
            Some(checkin) => now >= start_of_utc_day(checkin),
        },
        ShowWhen::BeforeCheckout => match checkout_at {
            None => true,
            Some(checkout) => now >= checkout - Duration::hours(48),
        },
        ShowWhen::CheckoutDay => match checkout_at {
            None => false,
            Some(checkout) => now >= start_of_utc_day(checkout),
        },
    }
}

fn start_of_utc_day(instant: DateTime<Utc>) -> DateTime<Utc> {
    let date = instant.date_naive();
    Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(raw: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(raw)
            .expect("rfc3339")
            .with_timezone(&Utc)
    }

    #[test]
    fn always_available() {
        assert!(is_checklist_available(
            ShowWhen::Always,
            utc("2026-07-01T10:00:00Z"),
            None,
            None
        ));
    }

    #[test]
    fn from_checkin_opens_on_checkin_day() {
        let checkin = utc("2026-07-20T15:00:00Z");
        assert!(!is_checklist_available(
            ShowWhen::FromCheckin,
            utc("2026-07-19T23:59:00Z"),
            Some(checkin),
            None
        ));
        assert!(is_checklist_available(
            ShowWhen::FromCheckin,
            utc("2026-07-20T00:00:00Z"),
            Some(checkin),
            None
        ));
    }

    #[test]
    fn before_checkout_opens_48h_prior() {
        let checkout = utc("2026-07-22T11:00:00Z");
        assert!(!is_checklist_available(
            ShowWhen::BeforeCheckout,
            utc("2026-07-20T10:59:00Z"),
            None,
            Some(checkout)
        ));
        assert!(is_checklist_available(
            ShowWhen::BeforeCheckout,
            utc("2026-07-20T11:00:00Z"),
            None,
            Some(checkout)
        ));
    }

    #[test]
    fn checkout_day_opens_on_checkout_day() {
        let checkout = utc("2026-07-22T11:00:00Z");
        assert!(!is_checklist_available(
            ShowWhen::CheckoutDay,
            utc("2026-07-21T23:59:00Z"),
            None,
            Some(checkout)
        ));
        assert!(is_checklist_available(
            ShowWhen::CheckoutDay,
            utc("2026-07-22T00:00:00Z"),
            None,
            Some(checkout)
        ));
    }
}
