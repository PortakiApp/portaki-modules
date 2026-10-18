//! Timed secret reveal — computed entirely inside the module.
//!
//! ## Fail-safe (missing stay / check-in)
//!
//! - [`RevealPolicy::Always`] → secrets are revealable even without check-in.
//! - Any other policy → **not revealable** until `checkin_at` is present and
//!   `now >= reveal_at(policy, checkin, timezone)`.
//! - Whatever the policy → **not revealable** once `now > checkout_at`.
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

/// Placeholder written into SDUI when a secret is not yet revealable.
/// Never put plaintext codes in the tree while locked.
pub const SECRET_MASK: &str = "••••••";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevealDecision {
    pub revealed: bool,
    /// UTC instant when secrets become available (`None` if unknown / always).
    pub available_from: Option<DateTime<Utc>>,
    /// The stay is over: nothing is revealed any more, whatever the policy.
    pub ended: bool,
}

/// Decide whether secrets may be shown in guest SDUI. `custom_hours` is read for
/// [`RevealPolicy::Custom`] only.
pub fn evaluate_reveal(
    policy: RevealPolicy,
    custom_hours: u32,
    now: DateTime<Utc>,
    checkin_at: Option<DateTime<Utc>>,
    checkout_at: Option<DateTime<Utc>>,
    property_timezone: &str,
) -> RevealDecision {
    // Never after departure: a past guest must not read the next guest's code.
    if checkout_at.is_some_and(|checkout| now > checkout) {
        return RevealDecision {
            revealed: false,
            available_from: None,
            ended: true,
        };
    }
    if matches!(policy, RevealPolicy::Always) {
        return RevealDecision {
            revealed: true,
            available_from: None,
            ended: false,
        };
    }

    let Some(checkin) = checkin_at else {
        return RevealDecision {
            revealed: false,
            available_from: None,
            ended: false,
        };
    };

    let Some(available_from) = reveal_at(policy, custom_hours, checkin, property_timezone) else {
        return RevealDecision {
            revealed: false,
            available_from: None,
            ended: false,
        };
    };

    RevealDecision {
        revealed: now >= available_from,
        available_from: Some(available_from),
        ended: false,
    }
}

/// Instant from which secrets are visible for `policy`; `custom_hours` before check-in for
/// [`RevealPolicy::Custom`].
pub fn reveal_at(
    policy: RevealPolicy,
    custom_hours: u32,
    checkin_at: DateTime<Utc>,
    property_timezone: &str,
) -> Option<DateTime<Utc>> {
    match policy {
        RevealPolicy::Always => None,
        RevealPolicy::HoursBefore24 => Some(checkin_at - Duration::hours(24)),
        RevealPolicy::AtCheckin => Some(checkin_at),
        RevealPolicy::Custom => Some(checkin_at - Duration::hours(i64::from(custom_hours))),
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
    // Resolve offset at the candidate local time (DST-aware for EU zones).
    let utc_guess = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
    let offset = offset_for_iana(tz_name, utc_guess);
    match offset.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => Some(dt.with_timezone(&Utc)),
        chrono::LocalResult::Ambiguous(earliest, _) => Some(earliest.with_timezone(&Utc)),
        chrono::LocalResult::None => {
            // Gap (spring forward): try again with offset computed from naive-as-UTC ± 1h.
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

/// `available_from` à l'heure du logement, écrit en toutes lettres : « lundi 19 octobre à 16 h ».
///
/// Jamais « 19/10/2026 à 16:00 » : un voyageur âgé lit mal une date en chiffres, et l'ordre
/// jour/mois change d'un pays à l'autre. Les mots viennent des bundles du module, qui parlent les
/// dix langues du livret — les tables du SDK n'en parlent que six.
pub fn format_available_from(available_from: DateTime<Utc>, property_timezone: &str) -> String {
    written_date(available_from.with_timezone(&offset_for_iana(property_timezone, available_from)))
}

/// [`format_available_from`], mais « aujourd'hui à 16 h » ou « demain à 16 h » quand c'est le cas,
/// vu de `now` à l'heure du logement. Pour le livret seulement : un e-mail se lit plus tard.
pub fn format_available_from_near(
    available_from: DateTime<Utc>,
    now: DateTime<Utc>,
    property_timezone: &str,
) -> String {
    use portaki_sdk::host::time::{relative_day, RelativeDay};

    let local = available_from.with_timezone(&offset_for_iana(property_timezone, available_from));
    let today = now.with_timezone(&offset_for_iana(property_timezone, now));
    let key = match relative_day(local.naive_local(), today.naive_local()) {
        RelativeDay::Today | RelativeDay::Tonight => "reveal.availableFrom.today",
        RelativeDay::Tomorrow => "reveal.availableFrom.tomorrow",
        // Plus loin, ou un cas que le SDK ajouterait : la date entière, qui reste juste.
        _ => return written_date(local),
    };
    word(key, &[("time", &clock(local))]).unwrap_or_else(|| written_date(local))
}

/// « lundi 19 octobre à 16 h », « Monday, October 19 at 4 pm ».
fn written_date(local: DateTime<FixedOffset>) -> String {
    const WEEKDAYS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
    let weekday = word(
        &format!(
            "date.weekday.{}",
            WEEKDAYS[local.weekday().num_days_from_monday() as usize]
        ),
        &[],
    );
    let month = word(&format!("date.month.{}", local.month()), &[]);
    let day = match local.day() {
        1 => word("date.firstOfMonth", &[]),
        day => Some(day.to_string()),
    };
    let time = clock(local);
    let fallback = || {
        format!(
            "{:02}/{:02}/{} {:02}:{:02}",
            local.day(),
            local.month(),
            local.year(),
            local.hour(),
            local.minute()
        )
    };
    let (Some(weekday), Some(month), Some(day)) = (weekday, month, day) else {
        return fallback();
    };
    word(
        "reveal.availableFrom.datetime",
        &[
            ("weekday", &weekday),
            ("day", &day),
            ("month", &month),
            ("time", &time),
        ],
    )
    .unwrap_or_else(fallback)
}

/// L'heure comme on la dit : « 16 h », « 16 h 30 », « 4 pm », « 16:30 » selon la langue.
pub fn clock(at: impl Timelike) -> String {
    let (hour, minute) = (at.hour(), at.minute());
    let key = if minute == 0 {
        "time.onTheHour"
    } else {
        "time.withMinutes"
    };
    let hour12 = match hour % 12 {
        0 => 12,
        h => h,
    };
    word(
        key,
        &[
            ("hour", &hour.to_string()),
            ("hour2", &format!("{hour:02}")),
            ("minute", &format!("{minute:02}")),
            ("hour12", &hour12.to_string()),
            ("ampm", if hour < 12 { "am" } else { "pm" }),
        ],
    )
    .unwrap_or_else(|| format!("{hour:02}:{minute:02}"))
}

/// Un texte du bundle, `None` quand la clé manque — l'hôte la renvoie alors telle quelle.
fn word(key: &str, vars: &[(&str, &str)]) -> Option<String> {
    let mut bag = portaki_sdk::host::i18n::Vars::new();
    for (name, value) in vars {
        bag.set(*name, *value);
    }
    portaki_sdk::host::i18n::translate(key, &bag)
        .ok()
        .filter(|text| text != key)
}

/// Human message when secrets are locked.
pub fn locked_message(available_from_label: Option<&str>) -> String {
    match available_from_label {
        Some(when) => t!("reveal.locked.withWhen", when = when)
            .unwrap_or_else(|_| format!("Available from {when}.")),
        None => t!("reveal.locked.generic")
            .unwrap_or_else(|_| "Codes will be available closer to check-in.".into()),
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // tests natifs : l'horloge du système y est disponible
mod tests {
    use super::*;

    /// Le bundle fr du module, pour lire la date comme le voyageur.
    fn fr() -> portaki_test_utils::MockContextBuilder {
        let bundle: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(include_str!("../i18n/fr-FR.json")).expect("fr bundle");
        bundle
            .into_iter()
            .fold(portaki_test_utils::MockContext::guest(), |b, (k, v)| {
                b.with_translation(&k, v.as_str().unwrap_or_default())
            })
    }

    #[test]
    #[serial_test::serial]
    fn the_reveal_date_reads_in_words() {
        fr().run(|_| {
            // Lundi 19 octobre 2026, 16 h à Paris (CEST).
            let at = utc("2026-10-19T14:00:00Z");
            let tz = "Europe/Paris";
            assert_eq!(
                format_available_from_near(at, utc("2026-10-10T08:00:00Z"), tz),
                "lundi 19 octobre à 16 h"
            );
            assert_eq!(
                format_available_from_near(at, utc("2026-10-18T20:00:00Z"), tz),
                "demain à 16 h"
            );
            assert_eq!(
                format_available_from_near(at, utc("2026-10-19T06:00:00Z"), tz),
                "aujourd’hui à 16 h"
            );
            // Le 1er, et des minutes.
            assert_eq!(
                format_available_from_near(
                    utc("2026-06-01T14:30:00Z"),
                    utc("2026-05-01T08:00:00Z"),
                    tz
                ),
                "lundi 1er juin à 16 h 30"
            );
        });
    }

    fn utc(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s)
            .expect("rfc3339")
            .with_timezone(&Utc)
    }

    #[test]
    fn always_reveals_without_checkin() {
        let d = evaluate_reveal(
            RevealPolicy::Always,
            24,
            Utc::now(),
            None,
            None,
            "Europe/Paris",
        );
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
            let d = evaluate_reveal(policy, 24, Utc::now(), None, None, "Europe/Paris");
            assert!(!d.revealed, "{policy:?}");
            assert!(d.available_from.is_none());
        }
    }

    #[test]
    fn hours_before_24() {
        let checkin = utc("2026-07-20T14:00:00Z");
        let at = reveal_at(RevealPolicy::HoursBefore24, 24, checkin, "Europe/Paris").unwrap();
        assert_eq!(at, utc("2026-07-19T14:00:00Z"));
        assert!(
            evaluate_reveal(
                RevealPolicy::HoursBefore24,
                24,
                utc("2026-07-19T14:00:00Z"),
                Some(checkin),
                None,
                "Europe/Paris",
            )
            .revealed
        );
        assert!(
            !evaluate_reveal(
                RevealPolicy::HoursBefore24,
                24,
                utc("2026-07-19T13:59:59Z"),
                Some(checkin),
                None,
                "Europe/Paris",
            )
            .revealed
        );
    }

    #[test]
    fn day_before_16h_paris_summer() {
        // Check-in Monday 20 Jul 2026 16:00 CEST (= 14:00 UTC) → reveal Sun 19 Jul 16:00 CEST (= 14:00 UTC)
        let checkin = utc("2026-07-20T14:00:00Z");
        let at = reveal_at(RevealPolicy::DayBefore16h, 24, checkin, "Europe/Paris").unwrap();
        assert_eq!(at, utc("2026-07-19T14:00:00Z"));
    }

    #[test]
    fn day_before_16h_paris_winter() {
        // Check-in Mon 19 Jan 2026 15:00 CET (= 14:00 UTC) → reveal Sun 18 Jan 16:00 CET (= 15:00 UTC)
        let checkin = utc("2026-01-19T14:00:00Z");
        let at = reveal_at(RevealPolicy::DayBefore16h, 24, checkin, "Europe/Paris").unwrap();
        assert_eq!(at, utc("2026-01-18T15:00:00Z"));
    }

    #[test]
    fn at_checkin() {
        let checkin = utc("2026-07-20T14:00:00Z");
        assert_eq!(
            reveal_at(RevealPolicy::AtCheckin, 24, checkin, "Europe/Paris"),
            Some(checkin)
        );
    }

    #[test]
    fn custom_hours_before_checkin() {
        let checkin = utc("2026-07-20T14:00:00Z");
        assert_eq!(
            reveal_at(RevealPolicy::Custom, 6, checkin, "Europe/Paris"),
            Some(utc("2026-07-20T08:00:00Z"))
        );
        let at = |now| {
            evaluate_reveal(
                RevealPolicy::Custom,
                168,
                utc(now),
                Some(checkin),
                None,
                "UTC",
            )
        };
        assert!(!at("2026-07-13T13:59:59Z").revealed);
        assert!(at("2026-07-13T14:00:00Z").revealed);
    }

    #[test]
    fn nothing_is_revealed_after_checkout() {
        let checkin = utc("2026-07-20T14:00:00Z");
        let checkout = utc("2026-07-25T10:00:00Z");
        for policy in [
            RevealPolicy::Always,
            RevealPolicy::HoursBefore24,
            RevealPolicy::DayBefore16h,
            RevealPolicy::AtCheckin,
        ] {
            let at =
                |now| evaluate_reveal(policy, 24, utc(now), Some(checkin), Some(checkout), "UTC");
            assert!(at("2026-07-25T10:00:00Z").revealed, "{policy:?}");
            let after = at("2026-07-25T10:00:01Z");
            assert!(!after.revealed && after.ended, "{policy:?}");
        }
    }
}
