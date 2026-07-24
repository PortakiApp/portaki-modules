//! Guest form availability from `ModuleConfig.show_when` + stay check-in.

use chrono::{DateTime, Duration, TimeZone, Utc};

use crate::config::ShowWhen;

/// Whether the guest form should be shown right now.
pub fn is_form_available(
    policy: ShowWhen,
    now: DateTime<Utc>,
    checkin_at: Option<DateTime<Utc>>,
) -> bool {
    match policy {
        ShowWhen::Confirm => true,
        ShowWhen::Before => match checkin_at {
            // No check-in yet — fail open so the form is still reachable.
            None => true,
            Some(checkin) => now >= checkin - Duration::hours(48),
        },
        ShowWhen::Checkin => match checkin_at {
            None => false,
            Some(checkin) => now >= start_of_utc_day(checkin),
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
    fn confirm_always_available() {
        assert!(is_form_available(ShowWhen::Confirm, utc("2026-07-01T10:00:00Z"), None));
    }

    #[test]
    fn before_opens_48h_prior() {
        let checkin = utc("2026-07-20T14:00:00Z");
        assert!(!is_form_available(
            ShowWhen::Before,
            utc("2026-07-18T13:59:00Z"),
            Some(checkin)
        ));
        assert!(is_form_available(
            ShowWhen::Before,
            utc("2026-07-18T14:00:00Z"),
            Some(checkin)
        ));
    }

    #[test]
    fn checkin_opens_on_checkin_day() {
        let checkin = utc("2026-07-20T14:00:00Z");
        assert!(!is_form_available(
            ShowWhen::Checkin,
            utc("2026-07-19T23:59:00Z"),
            Some(checkin)
        ));
        assert!(is_form_available(
            ShowWhen::Checkin,
            utc("2026-07-20T00:00:00Z"),
            Some(checkin)
        ));
    }
}
