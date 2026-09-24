//! Guest checklist availability from the list trigger + stay window.

use chrono::{DateTime, Duration, TimeZone, Utc};

use crate::lists;

/// Whether a guest list with `trigger` should be shown right now.
///
/// `beforeArrival` shows from the booking, `duringStay` from the check-in day, `atDeparture`
/// 48 h before check-out. A missing stay date fails open.
pub fn is_checklist_available(
    trigger: &str,
    now: DateTime<Utc>,
    checkin_at: Option<DateTime<Utc>>,
    checkout_at: Option<DateTime<Utc>>,
) -> bool {
    match trigger {
        lists::DURING_STAY => checkin_at.is_none_or(|checkin| now >= start_of_utc_day(checkin)),
        lists::AT_DEPARTURE => {
            checkout_at.is_none_or(|checkout| now >= checkout - Duration::hours(48))
        }
        _ => true,
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
    fn before_arrival_always_available() {
        assert!(is_checklist_available(
            lists::BEFORE_ARRIVAL,
            utc("2026-07-01T10:00:00Z"),
            None,
            None
        ));
    }

    #[test]
    fn during_stay_opens_on_checkin_day() {
        let checkin = Some(utc("2026-07-20T15:00:00Z"));
        let during = lists::DURING_STAY;
        assert!(!is_checklist_available(
            during,
            utc("2026-07-19T23:59:00Z"),
            checkin,
            None
        ));
        assert!(is_checklist_available(
            during,
            utc("2026-07-20T00:00:00Z"),
            checkin,
            None
        ));
    }

    #[test]
    fn at_departure_opens_48h_prior() {
        let checkout = Some(utc("2026-07-22T11:00:00Z"));
        let departure = lists::AT_DEPARTURE;
        assert!(!is_checklist_available(
            departure,
            utc("2026-07-20T10:59:00Z"),
            None,
            checkout
        ));
        assert!(is_checklist_available(
            departure,
            utc("2026-07-20T11:00:00Z"),
            None,
            checkout
        ));
    }
}
