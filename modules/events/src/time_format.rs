//! Event datetime parsing and guest display labels.

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use portaki_sdk::prelude::*;

use portaki_sdk::context::StayContext;

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

/// La fenêtre du séjour, élargie d'un jour de chaque côté (§0.7).
///
/// « Rien avant, rien après » : un séjour de trois nuits n'a pas à montrer le festival du mois
/// prochain, et la liste se remplissait pourtant de tout ce que l'agenda avait à dire. La veille
/// de l'arrivée et le lendemain du départ restent dedans — on prépare la veille, et on part après
/// le petit-déjeuner.
///
/// Sans dates de séjour — un aperçu, une surface hors séjour — rien n'est écarté : mieux vaut la
/// liste entière qu'une fenêtre inventée autour d'une date qu'on n'a pas.
pub fn stay_window(stay: Option<&StayContext>) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let stay = stay?;
    let from = stay.checkin_at? - Duration::days(1);
    let to = stay.checkout_at? + Duration::days(1);
    (from <= to).then_some((from, to))
}

/// Les événements qui tombent dans cette fenêtre, dans l'ordre reçu.
///
/// Un événement sans date lisible est gardé : il vient de l'hôte, qui a écrit quelque chose, et
/// l'écarter sur une date qu'on n'a pas su lire effacerait son travail.
pub fn events_within(
    events: &[EventRow],
    window: Option<(DateTime<Utc>, DateTime<Utc>)>,
) -> Vec<EventRow> {
    let Some((from, to)) = window else {
        return events.to_vec();
    };
    events
        .iter()
        // Un événement qui se répète tombe dans la fenêtre à sa prochaine occurrence.
        .map(|event| event.next_from(from))
        .filter(|event| match parse_starts_at(&event.starts_at) {
            Some(at) => at >= from && at <= to,
            None => true,
        })
        .collect()
}

/// Filters out past events for the home card.
///
/// `now` is passed in from the host context (via `time::now()`); calling `Utc::now()` here would
/// panic in the Wasm sandbox, which has no wall clock.
pub fn events_for_home_card(events: &[EventRow], now: DateTime<Utc>) -> Vec<EventRow> {
    let parsed: Vec<(EventRow, Option<DateTime<Utc>>)> = events
        .iter()
        .map(|e| e.next_from(now))
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

/// Le badge « quand » d'un événement : annulé d'abord, puis toute la journée, sinon l'heure.
pub fn event_when(event: &EventRow) -> String {
    if event.cancelled {
        return "i18n:guest.event.cancelled".to_string();
    }
    if event.all_day {
        return "i18n:guest.event.allDay".to_string();
    }
    format_starts_at_display(&event.starts_at)
}

pub fn format_starts_at_display(raw: &str) -> String {
    let Some(at) = parse_starts_at(raw) else {
        return raw.trim().to_string();
    };
    let time = at.format("%H:%M").to_string();
    t!("guest.event.startsAt", time = &time).unwrap_or_else(|_| time)
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
                ..Default::default()
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
                ..Default::default()
            },
        ];
        let now = Utc.with_ymd_and_hms(2050, 1, 1, 0, 0, 0).unwrap();
        let filtered = events_for_home_card(&events, now);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "b");
    }

    /// §0.7 : de la veille de l'arrivée au lendemain du départ, et rien au-delà.
    #[test]
    fn the_window_opens_the_day_before_and_closes_the_day_after() {
        let stay = StayContext {
            checkin_at: Some(at("2026-08-10T15:00:00Z")),
            checkout_at: Some(at("2026-08-13T10:00:00Z")),
            ..StayContext::default()
        };
        let (from, to) = stay_window(Some(&stay)).expect("fenêtre");
        assert_eq!(from, at("2026-08-09T15:00:00Z"));
        assert_eq!(to, at("2026-08-14T10:00:00Z"));

        let events = [
            row("avant", "2026-08-01T20:00:00Z"),
            row("la veille", "2026-08-09T20:00:00Z"),
            row("pendant", "2026-08-11T20:00:00Z"),
            row("le lendemain", "2026-08-14T09:00:00Z"),
            row("le mois prochain", "2026-09-15T20:00:00Z"),
            row("sans date", ""),
        ];
        let kept: Vec<String> = events_within(&events, Some((from, to)))
            .into_iter()
            .map(|e| e.title.get("fr").to_string())
            .collect();
        assert_eq!(kept, ["la veille", "pendant", "le lendemain", "sans date"]);
    }

    /// Sans dates de séjour, rien n'est écarté : mieux vaut la liste entière qu'une fenêtre
    /// inventée autour d'une date qu'on n'a pas.
    #[test]
    fn no_stay_no_window() {
        assert_eq!(stay_window(None), None);
        let events = [row("un jour", "2027-01-01T20:00:00Z")];
        assert_eq!(events_within(&events, None).len(), 1);
    }

    fn at(raw: &str) -> DateTime<Utc> {
        parse_starts_at(raw).expect("date")
    }

    fn row(title: &str, starts_at: &str) -> EventRow {
        EventRow {
            title: portaki_sdk::contracts::i18n::I18nText::new(title, title),
            starts_at: starts_at.to_string(),
            ..EventRow::default()
        }
    }
}
