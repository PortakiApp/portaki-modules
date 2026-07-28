//! Event datetime parsing and guest display labels.

use chrono::{DateTime, NaiveDateTime, Utc};
use portaki_sdk::prelude::*;

use crate::config::EventRow;

pub fn parse_starts_at(raw: &str) -> Option<DateTime<Utc>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = trimmed.parse::<DateTime<Utc>>() {
        return Some(dt);
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S") {
        return Some(naive.and_utc());
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S") {
        return Some(naive.and_utc());
    }
    None
}

/// Filters out past events for the home card.
///
/// `now` is passed in from the host context (via `time::now()`); calling `Utc::now()` here would
/// panic in the Wasm sandbox, which has no wall clock.
pub fn events_for_home_card(events: &[EventRow], now: DateTime<Utc>) -> Vec<EventRow> {
    let parsed: Vec<(EventRow, Option<DateTime<Utc>>)> = events
        .iter()
        .cloned()
        .map(|e| {
            let at = parse_starts_at(&e.starts_at);
            (e, at)
        })
        .collect();
    let any_parseable = parsed.iter().any(|(_, at)| at.is_some());
    if !any_parseable {
        return events.to_vec();
    }
    parsed
        .into_iter()
        .filter_map(|(event, at)| {
            let at = at?;
            if at >= now {
                Some(event)
            } else {
                None
            }
        })
        .collect()
}

pub fn sort_events_by_start(mut events: Vec<EventRow>) -> Vec<EventRow> {
    events.sort_by(|left, right| {
        let left_at = parse_starts_at(&left.starts_at);
        let right_at = parse_starts_at(&right.starts_at);
        match (left_at, right_at) {
            (Some(l), Some(r)) => l.cmp(&r),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => left.id.cmp(&right.id),
        }
    });
    events
}

pub fn day_badge_label(at: DateTime<Utc>) -> String {
    let weekday_key = weekday_i18n_key(at);
    let weekday = t!(&weekday_key).unwrap_or_else(|_| weekday_key.clone());
    let day = at.format("%d").to_string();
    t!("guest.event.dayBadge", weekday = &weekday, day = &day)
        .unwrap_or_else(|_| format!("{weekday} {day}"))
}

pub fn format_starts_at_display(raw: &str) -> String {
    let Some(at) = parse_starts_at(raw) else {
        return raw.trim().to_string();
    };
    let time = at.format("%H:%M").to_string();
    t!("guest.event.startsAt", time = &time).unwrap_or_else(|_| time)
}

fn weekday_i18n_key(at: DateTime<Utc>) -> String {
    use chrono::Datelike;
    match at.weekday().number_from_monday() {
        1 => "guest.weekday.mon".to_string(),
        2 => "guest.weekday.tue".to_string(),
        3 => "guest.weekday.wed".to_string(),
        4 => "guest.weekday.thu".to_string(),
        5 => "guest.weekday.fri".to_string(),
        6 => "guest.weekday.sat".to_string(),
        _ => "guest.weekday.sun".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn home_card_filters_past_when_parseable() {
        let past = Utc.with_ymd_and_hms(2020, 1, 1, 12, 0, 0).unwrap();
        let future = Utc.with_ymd_and_hms(2099, 6, 1, 18, 0, 0).unwrap();
        let events = vec![
            EventRow {
                id: "a".into(),
                title: Default::default(),
                place: Default::default(),
                starts_at: past.to_rfc3339(),
                ends_at: None,
                url: None,
                lat: None,
                lng: None,
                note: None,
            },
            EventRow {
                id: "b".into(),
                title: Default::default(),
                place: Default::default(),
                starts_at: future.to_rfc3339(),
                ends_at: None,
                url: None,
                lat: None,
                lng: None,
                note: None,
            },
        ];
        let now = Utc.with_ymd_and_hms(2050, 1, 1, 0, 0, 0).unwrap();
        let filtered = events_for_home_card(&events, now);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "b");
    }
}
