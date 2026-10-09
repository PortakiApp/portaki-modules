//! « Conflit de dates · L'Islette · 21–24/08 » et « Flux Booking injoignable depuis 2 j »
//! (spec Calendriers §1, §9) : `timelineTasks`. Calculées, jamais stockées, sans case à cocher :
//! une tâche disparaît quand le conflit ou la panne cesse.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::contracts::timeline::{
    self, TimelineStay, TimelineTask, TimelineTasks, TimelineTasksArgs,
};
use portaki_sdk::host::time::{self, PropertyTz};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;
use crate::email_send::source_label;
use crate::i18n;
use crate::sync_state::{load_sync_state, SyncState};

/// Une panne plus courte n'est pas une tâche : la synchro suivante peut la lever.
const FEED_DOWN_AFTER_DAYS: i64 = 2;

#[portaki_sdk::query(
    name = "timelineTasks",
    example(
        label = "Semaine en cours",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","from":"2026-08-17T00:00:00Z","to":"2026-08-24T00:00:00Z","stays":[{"id":"0b6f2c3e-1d4a-4e5b-8c7d-9e0f1a2b3c4d","checkIn":"2026-08-21T14:00:00Z","checkOut":"2026-08-24T08:00:00Z","guestName":"Ada","status":"UPCOMING"},{"id":"1c7a3d4f-2e5b-4f6c-9d8e-0f1a2b3c4d5e","checkIn":"2026-08-22T14:00:00Z","checkOut":"2026-08-25T08:00:00Z","guestName":"Liam","status":"UPCOMING"}]}"#
    )
)]
pub fn timeline_tasks(ctx: Context, args: TimelineTasksArgs) -> Result<TimelineTasks> {
    let config = ModuleConfig::load(&ctx)?;
    let state = load_sync_state().unwrap_or_default();
    let tz = PropertyTz::parse(&ctx.property.timezone);
    let mut tasks = conflicts(&args, &ctx.property.name, tz.as_ref());
    tasks.extend(feeds_down(&args, &config, &state, time::now()?));
    tasks.sort_by_key(|task| task.at);
    Ok(TimelineTasks { tasks })
}

/// Chaque paire de séjours dont les nuits se chevauchent, posée au début du chevauchement. Les
/// séjours annulés n'arrivent pas jusqu'ici ; un départ le jour d'une arrivée n'est pas un conflit.
fn conflicts(
    args: &TimelineTasksArgs,
    property: &str,
    tz: Option<&PropertyTz>,
) -> Vec<TimelineTask> {
    let mut tasks = Vec::new();
    for (i, a) in args.stays.iter().enumerate() {
        for b in &args.stays[i + 1..] {
            let start = a.check_in.max(b.check_in);
            let end = a.check_out.min(b.check_out);
            if start >= end || start > args.to || end <= args.from {
                continue;
            }
            let context = format!("{property} · {}", dates(start, end, tz));
            tasks.push(
                timeline::task(
                    format!("conflict:{}:{}", a.id, b.id),
                    start.max(args.from),
                    args.property_id,
                    i18n::text("task.conflict.title", &[]),
                    I18nText::new(&context, &context),
                )
                .stay(later(a, b).id),
            );
        }
    }
    tasks
}

fn later<'a>(a: &'a TimelineStay, b: &'a TimelineStay) -> &'a TimelineStay {
    if b.check_in > a.check_in {
        b
    } else {
        a
    }
}

/// « 21–24/08 », « 30/08–02/09 » : les dates du logement.
fn dates(start: DateTime<Utc>, end: DateTime<Utc>, tz: Option<&PropertyTz>) -> String {
    let local = |at: DateTime<Utc>| -> NaiveDate {
        tz.map_or(at.date_naive(), |tz| tz.to_local(at).date_naive())
    };
    let (from, until) = (local(start), local(end));
    if from.format("%m").to_string() == until.format("%m").to_string() {
        format!("{}–{}", from.format("%d"), until.format("%d/%m"))
    } else {
        format!("{}–{}", from.format("%d/%m"), until.format("%d/%m"))
    }
}

/// Les flux relevés qui ne répondent plus depuis [`FEED_DOWN_AFTER_DAYS`] jours.
fn feeds_down(
    args: &TimelineTasksArgs,
    config: &ModuleConfig,
    state: &SyncState,
    now: DateTime<Utc>,
) -> Vec<TimelineTask> {
    config
        .connected_calendars()
        .into_iter()
        .filter_map(|feed| {
            let since = state.feed_failures.get(&feed.id)?.since.as_str();
            let since = DateTime::parse_from_rfc3339(since)
                .ok()?
                .with_timezone(&Utc);
            let late_at = since + Duration::days(FEED_DOWN_AFTER_DAYS);
            if late_at > now || late_at > args.to {
                return None;
            }
            let days = (now - since).num_days().to_string();
            let source = source_label(feed.format, feed.label.as_deref());
            Some(timeline::task(
                format!("feed-down:{}", feed.id),
                late_at.max(args.from),
                args.property_id,
                i18n::text(
                    "task.feedDown.title",
                    &[("source", &source), ("days", &days)],
                ),
                i18n::text("task.feedDown.context", &[]),
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::config::Config;

    fn at(raw: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(raw)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn stay(check_in: &str, check_out: &str) -> TimelineStay {
        TimelineStay {
            id: Uuid::new_v4(),
            check_in: at(check_in),
            check_out: at(check_out),
            guest_name: "Ada".into(),
            status: "UPCOMING".into(),
        }
    }

    fn args(stays: Vec<TimelineStay>) -> TimelineTasksArgs {
        TimelineTasksArgs {
            property_id: Uuid::nil(),
            from: at("2026-08-17T00:00:00Z"),
            to: at("2026-08-31T00:00:00Z"),
            stays,
        }
    }

    #[test]
    fn overlapping_stays_are_a_conflict_and_a_turnover_is_not() {
        let stays = vec![
            stay("2026-08-18T14:00:00Z", "2026-08-21T08:00:00Z"),
            stay("2026-08-21T14:00:00Z", "2026-08-24T08:00:00Z"), // même jour : enchaînement
            stay("2026-08-21T15:00:00Z", "2026-08-25T08:00:00Z"), // chevauche le précédent
        ];
        let tz = PropertyTz::parse("Europe/Paris");
        let tasks = conflicts(&args(stays.clone()), "L'Islette", tz.as_ref());
        assert_eq!(tasks.len(), 1);
        let task = &tasks[0];
        assert_eq!(task.id, format!("conflict:{}:{}", stays[1].id, stays[2].id));
        assert_eq!(task.stay_id, Some(stays[2].id));
        assert_eq!(task.at, at("2026-08-21T15:00:00Z"));
        assert_eq!(task.context.fr, "L'Islette · 21–24/08");
        assert_eq!(task.title.fr, "Conflit de dates");
    }

    #[test]
    fn a_conflict_across_months_spells_both() {
        assert_eq!(
            dates(at("2026-08-30T14:00:00Z"), at("2026-09-02T08:00:00Z"), None),
            "30/08–02/09"
        );
    }

    #[test]
    fn a_feed_down_for_two_days_becomes_a_task() {
        let config: Config = serde_json::from_value(json!({ "calendars": [
            { "id": "booking", "url": "https://admin.booking.com/hotel/ical.html?t=x", "channel": "booking" },
            { "id": "airbnb", "url": "https://www.airbnb.com/calendar/ical/1.ics" },
        ]}))
        .unwrap();
        let config = ModuleConfig {
            calendars: crate::config::feeds(&config.calendars),
        };
        let mut state = SyncState::default();
        state.record_feed("booking", false, "2026-08-18T06:00:00Z");
        state.record_feed("airbnb", false, "2026-08-19T18:00:00Z");

        let tasks = feeds_down(&args(vec![]), &config, &state, at("2026-08-20T10:00:00Z"));
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "feed-down:booking");
        assert_eq!(tasks[0].at, at("2026-08-20T06:00:00Z"));
        assert_eq!(tasks[0].title.fr, "Flux Booking injoignable depuis 2 j");
    }
}
