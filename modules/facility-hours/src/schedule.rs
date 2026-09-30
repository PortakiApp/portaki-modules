//! Structured opening hours, and what they say right now (§2.6).
//!
//! A row used to carry one free-text line — `"08:00 – 20:00"`, `"à partir de 16:00"`, `"24 h/24"`.
//! That reads fine and computes nothing: "Ouvert", "Ouvre à 08:00" and "Fermé" cannot be derived
//! from a sentence, and neither can Monday-to-Sunday.
//!
//! So a row may now carry times as well. **May**, not must: hosts have already written their hours
//! in prose, and nothing here throws that away. A row without structured times keeps showing its
//! sentence exactly as before, and gains no live state — which is honest, rather than a guess at
//! what "à partir de 16:00" means.
//!
//! Times are `HH:MM` in the property's timezone. A closing time earlier than its opening spans
//! midnight: `22:00 – 02:00` is open at one in the morning, and saying otherwise would put a guest
//! outside a door that is in fact open.

use portaki_sdk::host::time::PropertyTz;
use serde::{Deserialize, Serialize};

use chrono::{DateTime, Datelike, Timelike, Utc, Weekday};

/// Minutes since midnight, or `None` when the text is not `HH:MM`.
///
/// Deliberately strict: a lenient parser would read "à partir de 16:00" as 16:00 and drop the
/// "à partir de", turning a sentence the host wrote into a schedule they did not.
pub fn parse_hm(raw: &str) -> Option<u32> {
    let text = raw.trim();
    let (hours, minutes) = text.split_once(':')?;
    if hours.len() > 2 || minutes.len() != 2 {
        return None;
    }
    let hours: u32 = hours.parse().ok()?;
    let minutes: u32 = minutes.parse().ok()?;
    if hours > 23 || minutes > 59 {
        return None;
    }
    Some(hours * 60 + minutes)
}

/// One day's exception — different hours, or closed (§2.6).
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DayHours {
    /// `mon` … `sun`.
    pub day: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opens_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closes_at: Option<String>,
    /// Closed that day, whatever the times say.
    pub closed: bool,
}

/// The seven days, in the order a week is read.
pub const WEEK: [Weekday; 7] = [
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
    Weekday::Sat,
    Weekday::Sun,
];

/// The `mon` … `sun` key for a weekday.
pub fn day_key(day: Weekday) -> &'static str {
    match day {
        Weekday::Mon => "mon",
        Weekday::Tue => "tue",
        Weekday::Wed => "wed",
        Weekday::Thu => "thu",
        Weekday::Fri => "fri",
        Weekday::Sat => "sat",
        Weekday::Sun => "sun",
    }
}

/// What a facility is doing at a given moment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// Open around the clock — no times to compare.
    AlwaysOpen,
    Open,
    /// Shut for now; opens at this many minutes past midnight, today.
    OpensAt(u32),
    Closed,
}

/// The opening and closing of one day, once exceptions are applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DaySpan {
    pub opens: u32,
    pub closes: u32,
}

impl DaySpan {
    /// The span runs past midnight (`22:00 – 02:00`).
    pub fn overnight(&self) -> bool {
        self.closes <= self.opens
    }

    /// `minutes` falls inside the span, midnight crossing included.
    pub fn covers(&self, minutes: u32) -> bool {
        if self.overnight() {
            minutes >= self.opens || minutes < self.closes
        } else {
            minutes >= self.opens && minutes < self.closes
        }
    }
}

/// The hours a facility keeps, as the host filled them in.
#[derive(Debug, Clone, Default)]
pub struct Schedule {
    pub all_day: bool,
    pub opens_at: Option<u32>,
    pub closes_at: Option<u32>,
    pub exceptions: Vec<DayHours>,
}

impl Schedule {
    /// Whether anything here can be computed. A row without times keeps its sentence.
    pub fn is_structured(&self) -> bool {
        self.all_day || (self.opens_at.is_some() && self.closes_at.is_some())
    }

    /// The span for one weekday, or `None` when that day is closed.
    pub fn span_on(&self, day: Weekday) -> Option<DaySpan> {
        let key = day_key(day);
        let exception = self
            .exceptions
            .iter()
            .find(|entry| entry.day.trim().eq_ignore_ascii_case(key));

        if let Some(exception) = exception {
            if exception.closed {
                return None;
            }
            let opens = exception.opens_at.as_deref().and_then(parse_hm);
            let closes = exception.closes_at.as_deref().and_then(parse_hm);
            if let (Some(opens), Some(closes)) = (opens, closes) {
                return Some(DaySpan { opens, closes });
            }
            // An exception that names neither time only says "not closed": the usual hours apply.
        }

        match (self.opens_at, self.closes_at) {
            (Some(opens), Some(closes)) => Some(DaySpan { opens, closes }),
            _ => None,
        }
    }

    /// What the facility is doing at `now`, read in the property's timezone.
    ///
    /// Yesterday is consulted before today: at one in the morning, a `22:00 – 02:00` facility is
    /// still inside *yesterday's* span, and looking only at today would send the guest away from an
    /// open door.
    pub fn state_at(&self, now: DateTime<Utc>, tz: Option<&PropertyTz>) -> Option<State> {
        if self.all_day {
            return Some(State::AlwaysOpen);
        }
        if !self.is_structured() {
            return None;
        }

        let local = match tz {
            Some(tz) => tz.to_local(now).naive_local(),
            None => now.naive_utc(),
        };
        let minutes = local.hour() * 60 + local.minute();
        let today = local.weekday();

        if let Some(span) = self.span_on(today.pred()) {
            if span.overnight() && minutes < span.closes {
                return Some(State::Open);
            }
        }

        let Some(span) = self.span_on(today) else {
            return Some(State::Closed);
        };
        if span.covers(minutes) {
            return Some(State::Open);
        }
        if minutes < span.opens {
            return Some(State::OpensAt(span.opens));
        }
        Some(State::Closed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(hour: u32, minute: u32) -> DateTime<Utc> {
        // A Wednesday.
        Utc.with_ymd_and_hms(2026, 9, 30, hour, minute, 0).unwrap()
    }

    fn day(opens: &str, closes: &str) -> Schedule {
        Schedule {
            all_day: false,
            opens_at: parse_hm(opens),
            closes_at: parse_hm(closes),
            exceptions: Vec::new(),
        }
    }

    #[test]
    fn a_time_is_read_only_when_it_is_one() {
        assert_eq!(parse_hm("08:00"), Some(480));
        assert_eq!(parse_hm(" 8:05 "), Some(485));
        assert_eq!(parse_hm("23:59"), Some(1439));
        // The prose a host may have written stays prose.
        assert_eq!(parse_hm("à partir de 16:00"), None);
        assert_eq!(parse_hm("24 h/24"), None);
        assert_eq!(parse_hm("8h"), None);
        assert_eq!(parse_hm("24:00"), None);
        assert_eq!(parse_hm("08:60"), None);
    }

    #[test]
    fn a_row_without_times_computes_nothing() {
        let prose = Schedule::default();
        assert!(!prose.is_structured());
        assert_eq!(prose.state_at(at(10, 0), None), None);
    }

    #[test]
    fn open_before_closing_and_shut_after() {
        let pool = day("08:00", "20:00");
        assert_eq!(pool.state_at(at(7, 0), None), Some(State::OpensAt(480)));
        assert_eq!(pool.state_at(at(8, 0), None), Some(State::Open));
        assert_eq!(pool.state_at(at(19, 59), None), Some(State::Open));
        assert_eq!(pool.state_at(at(20, 0), None), Some(State::Closed));
    }

    #[test]
    fn a_bar_that_closes_after_midnight_is_open_at_one() {
        let bar = day("22:00", "02:00");
        assert_eq!(bar.state_at(at(23, 0), None), Some(State::Open));
        // One in the morning belongs to yesterday's span, not to a day that has not opened.
        assert_eq!(bar.state_at(at(1, 0), None), Some(State::Open));
        // À trois heures il est fermé, et il rouvre ce soir : « Ouvre à 22:00 » en dit plus que
        // « Fermé », et c'est ce que le §2.6 veut afficher.
        assert_eq!(bar.state_at(at(3, 0), None), Some(State::OpensAt(1320)));
    }

    #[test]
    fn a_closed_day_is_closed_whatever_the_usual_hours() {
        let mut gym = day("06:00", "22:00");
        gym.exceptions.push(DayHours {
            day: "wed".into(),
            closed: true,
            ..DayHours::default()
        });
        assert_eq!(gym.state_at(at(10, 0), None), Some(State::Closed));
        assert!(gym.span_on(Weekday::Wed).is_none());
        assert!(gym.span_on(Weekday::Thu).is_some());
    }

    #[test]
    fn a_day_may_keep_its_own_hours() {
        let mut spa = day("10:00", "19:00");
        spa.exceptions.push(DayHours {
            day: "WED".into(),
            opens_at: Some("14:00".into()),
            closes_at: Some("18:00".into()),
            closed: false,
        });
        // Case is the host's business, not the lookup's.
        assert_eq!(spa.state_at(at(11, 0), None), Some(State::OpensAt(840)));
        assert_eq!(spa.state_at(at(15, 0), None), Some(State::Open));
    }

    #[test]
    fn around_the_clock_needs_no_times() {
        let parking = Schedule {
            all_day: true,
            ..Schedule::default()
        };
        assert!(parking.is_structured());
        assert_eq!(parking.state_at(at(3, 0), None), Some(State::AlwaysOpen));
    }
}
