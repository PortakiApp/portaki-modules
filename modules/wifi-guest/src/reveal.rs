//! Timed password reveal — computed entirely inside the module.
//!
//! ## Fail-safe (missing stay / check-in)
//!
//! - [`RevealPolicy::Always`] → password is revealable even without check-in.
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

use serde_json::{Map, Value};

use crate::config::{ModuleConfig, RevealPolicy, DEFAULT_REVEAL_HOURS};

/// When the passwords open, and whose choice it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schedule {
    pub policy: RevealPolicy,
    /// « Personnalisé » : hours before check-in.
    pub hours: u32,
    pub source: Source,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// Wi-Fi's own `reveal_policy`.
    Own,
    /// The schedule a peer publishes (`access.reveal_schedule`), read-only in the drawer.
    Peer {
        module_id: String,
        display_name: String,
    },
}

/// A provider of the schedule, as the platform lists it: its id, its name, and the values it
/// shares (`ModulePeer.shared` — published values only, never a secret).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Peer {
    pub module_id: String,
    pub display_name: String,
    pub shared: Map<String, Value>,
}

impl Schedule {
    /// The peer's published `reveal_policy` / `reveal_hours` when it shares them, else Wi-Fi's
    /// own. A peer that shares nothing readable leaves Wi-Fi's own choice in force (spec §9 cas 8).
    pub fn resolve(config: &ModuleConfig, peer: Option<&Peer>) -> Self {
        let shared = peer.and_then(|peer| {
            let policy = peer.shared.get("reveal_policy")?.clone();
            let policy = serde_json::from_value::<RevealPolicy>(policy).ok()?;
            let hours = peer
                .shared
                .get("reveal_hours")
                .and_then(|h| h.as_u64().or_else(|| h.as_str()?.trim().parse().ok()))
                .and_then(|h| u32::try_from(h).ok());
            Some((peer, policy, hours))
        });
        match shared {
            Some((peer, policy, hours)) => Self {
                policy,
                hours: hours.unwrap_or(DEFAULT_REVEAL_HOURS),
                source: Source::Peer {
                    module_id: peer.module_id.clone(),
                    display_name: peer.display_name.clone(),
                },
            },
            None => Self {
                policy: config.reveal_policy,
                hours: config.reveal_hours.unwrap_or(DEFAULT_REVEAL_HOURS),
                source: Source::Own,
            },
        }
    }

    /// The schedule in force for this property.
    pub fn load(config: &ModuleConfig) -> Self {
        Self::resolve(config, reveal_schedule_peer().as_ref())
    }
}

/// The first module that provides `access.reveal_schedule` on this property, with the published
/// values it shares. A host error leaves Wi-Fi on its own choice: never block the booklet on it.
fn reveal_schedule_peer() -> Option<Peer> {
    let peer = host::module::list_by_capability(portaki_sdk::capability::access::REVEAL_SCHEDULE)
        .ok()?
        .into_iter()
        .next()?;
    Some(Peer {
        module_id: peer.module_id.to_string(),
        display_name: peer.display_name,
        shared: peer.shared,
    })
}

/// Placeholder written into SDUI when the password is not yet revealable.
pub const SECRET_MASK: &str = "••••••";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevealDecision {
    pub revealed: bool,
    pub available_from: Option<DateTime<Utc>>,
    /// The stay is over: nothing is revealed any more, whatever the policy.
    pub ended: bool,
}

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
        RevealPolicy::DayBefore16h => day_before_16h(checkin_at, property_timezone),
        RevealPolicy::Custom => Some(checkin_at - Duration::hours(i64::from(custom_hours))),
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

pub(crate) fn offset_for_iana(tz_name: &str, at: DateTime<Utc>) -> FixedOffset {
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

/// `available_from` à l'heure du logement, écrit en toutes lettres : « lundi 19 octobre à 16 h »,
/// ou « aujourd'hui à 16 h » / « demain à 16 h » vu de `now`.
///
/// Jamais « 19/10/2026 à 16:00 » : un voyageur âgé lit mal une date en chiffres, et l'ordre
/// jour/mois change d'un pays à l'autre. Les mots viennent des bundles du module, qui parlent les
/// dix langues du livret — les tables du SDK n'en parlent que six. Même écriture que
/// `access-guide` : les deux dates se lisent côte à côte dans le livret.
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
fn clock(at: impl Timelike) -> String {
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

pub fn locked_message(available_from_label: Option<&str>, locale: &str) -> String {
    let is_fr = locale.to_ascii_lowercase().starts_with("fr");
    match available_from_label {
        Some(when) => match t!("reveal.locked.withWhen", when = when) {
            Ok(text) if text.contains(when) && !looks_like_i18n_key(&text) => text,
            Ok(text) if text.contains("{when}") => text.replace("{when}", when),
            _ if is_fr => format!("Disponible dès {when}"),
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
    }

    #[test]
    fn day_before_16h_paris_summer() {
        let checkin = utc("2026-07-20T14:00:00Z");
        let at = reveal_at(RevealPolicy::DayBefore16h, 24, checkin, "Europe/Paris").unwrap();
        assert_eq!(at, utc("2026-07-19T14:00:00Z"));
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
            RevealPolicy::Custom,
        ] {
            let at =
                |now| evaluate_reveal(policy, 24, utc(now), Some(checkin), Some(checkout), "UTC");
            assert!(at("2026-07-25T10:00:00Z").revealed, "{policy:?}");
            let after = at("2026-07-25T10:00:01Z");
            assert!(!after.revealed && after.ended, "{policy:?}");
        }
    }

    #[test]
    fn custom_is_that_many_hours_before_checkin() {
        let checkin = utc("2026-07-20T14:00:00Z");
        assert_eq!(
            reveal_at(RevealPolicy::Custom, 6, checkin, "Europe/Paris"),
            Some(utc("2026-07-20T08:00:00Z"))
        );
    }

    fn peer(shared: serde_json::Value) -> Peer {
        Peer {
            module_id: "access".into(),
            display_name: "Accès".into(),
            shared: shared.as_object().cloned().unwrap_or_default(),
        }
    }

    /// Spec §2.2 / §9 cas 8 : le calendrier d'Accès quand il le partage, sinon celui du Wi-Fi.
    #[test]
    fn the_peer_schedule_wins_when_it_shares_one() {
        let config = ModuleConfig {
            reveal_policy: RevealPolicy::Always,
            ..ModuleConfig::default()
        };
        let own = Schedule::resolve(&config, None);
        assert_eq!(
            (own.policy, &own.source),
            (RevealPolicy::Always, &Source::Own)
        );

        let shared = Schedule::resolve(
            &config,
            Some(&peer(
                serde_json::json!({ "reveal_policy": "custom", "reveal_hours": 6 }),
            )),
        );
        assert_eq!((shared.policy, shared.hours), (RevealPolicy::Custom, 6));
        assert!(
            matches!(shared.source, Source::Peer { ref module_id, .. } if module_id == "access")
        );

        // L'ancienne graphie d'Accès se lit aussi ; un pair muet laisse le choix du Wi-Fi.
        let legacy = Schedule::resolve(
            &config,
            Some(&peer(
                serde_json::json!({ "reveal_policy": "day_before16h" }),
            )),
        );
        assert_eq!(legacy.policy, RevealPolicy::DayBefore16h);
        for silent in [
            serde_json::json!({}),
            serde_json::json!({ "reveal_policy": "weekly" }),
        ] {
            assert_eq!(Schedule::resolve(&config, Some(&peer(silent))), own);
        }
    }
}
