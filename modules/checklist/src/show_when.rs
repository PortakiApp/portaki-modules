//! Guest checklist availability from the list trigger + stay window.

use chrono::{DateTime, Days, NaiveTime, TimeZone, Utc};

use crate::guest::depart::offset_for_iana;
use crate::lists;

/// Whether a guest list with `trigger` should be shown right now (« Afficher », spec §2.1).
///
/// Days are counted in the property `timezone`: `duringStay` opens at midnight of the check-in
/// day, `atDeparture` (« La veille ») at midnight of the day before check-out, `departureDay` at
/// midnight of the check-out day. A missing stay date fails open.
pub fn is_checklist_available(
    trigger: &str,
    now: DateTime<Utc>,
    checkin_at: Option<DateTime<Utc>>,
    checkout_at: Option<DateTime<Utc>>,
    timezone: &str,
) -> bool {
    let opens = match trigger {
        lists::DURING_STAY => checkin_at.map(|checkin| start_of_day(checkin, 0, timezone)),
        lists::AT_DEPARTURE => checkout_at.map(|checkout| start_of_day(checkout, 1, timezone)),
        lists::DEPARTURE_DAY => checkout_at.map(|checkout| start_of_day(checkout, 0, timezone)),
        _ => None,
    };
    opens.is_none_or(|opens| now >= opens)
}

/// Midnight, in `timezone`, `days_before` days before the local day of `instant`.
fn start_of_day(instant: DateTime<Utc>, days_before: u64, timezone: &str) -> DateTime<Utc> {
    let offset = offset_for_iana(timezone, instant);
    let day = instant.with_timezone(&offset).date_naive() - Days::new(days_before);
    offset
        .from_local_datetime(&day.and_time(NaiveTime::MIN))
        .single()
        .map_or(instant, |local| local.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(raw: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(raw)
            .expect("rfc3339")
            .with_timezone(&Utc)
    }

    const PARIS: &str = "Europe/Paris";

    #[test]
    fn a_retired_or_unknown_trigger_is_always_available() {
        let now = utc("2026-07-01T10:00:00Z");
        assert!(is_checklist_available(
            lists::BEFORE_ARRIVAL,
            now,
            None,
            None,
            PARIS
        ));
    }

    #[test]
    fn during_stay_opens_on_checkin_day() {
        let checkin = Some(utc("2026-07-20T15:00:00Z"));
        let during = lists::DURING_STAY;
        // Midnight in Paris (UTC+2) is 22:00 UTC the day before.
        assert!(!is_checklist_available(
            during,
            utc("2026-07-19T21:59:00Z"),
            checkin,
            None,
            PARIS
        ));
        assert!(is_checklist_available(
            during,
            utc("2026-07-19T22:00:00Z"),
            checkin,
            None,
            PARIS
        ));
    }

    #[test]
    fn the_eve_opens_at_midnight_the_day_before_checkout() {
        let checkout = Some(utc("2026-07-22T09:00:00Z"));
        let eve = lists::AT_DEPARTURE;
        assert!(!is_checklist_available(
            eve,
            utc("2026-07-20T21:59:00Z"),
            None,
            checkout,
            PARIS
        ));
        assert!(is_checklist_available(
            eve,
            utc("2026-07-20T22:00:00Z"),
            None,
            checkout,
            PARIS
        ));
    }

    #[test]
    fn departure_day_opens_at_midnight_of_checkout_day() {
        let checkout = Some(utc("2026-07-22T09:00:00Z"));
        let day = lists::DEPARTURE_DAY;
        assert!(!is_checklist_available(
            day,
            utc("2026-07-21T21:59:00Z"),
            None,
            checkout,
            PARIS
        ));
        assert!(is_checklist_available(
            day,
            utc("2026-07-21T22:00:00Z"),
            None,
            checkout,
            PARIS
        ));
        assert!(is_checklist_available(
            day,
            utc("2026-07-22T00:00:00Z"),
            None,
            checkout,
            "UTC"
        ));
    }
}
