//! « Sortir les bacs · {logement} · ce soir » dans À venir (spec Tri §1) : `timelineTasks`,
//! `taskToggle`, `taskComplete`.
//!
//! Une tâche la veille de chaque collecte, quand l'hôte a choisi `host_reminder` : calculée,
//! jamais stockée. Son id porte la date de la collecte (`bins:2026-10-13`) ; seule la case
//! « Bacs sortis » cochée est gardée, en KV, par date. Le nom du logement, le dashboard l'ajoute
//! lui-même au titre.

use chrono::{DateTime, Datelike, Days, NaiveDate, Utc};
use portaki_sdk::contracts::timeline::{
    self, TaskCompleteArgs, TaskToggleArgs, TimelineTask, TimelineTaskItem, TimelineTasks,
    TimelineTasksArgs,
};
use portaki_sdk::host::kv;
use portaki_sdk::host::time::PropertyTz;
use portaki_sdk::prelude::*;

use crate::collection::parse_day;
use crate::config::ModuleConfig;
use crate::i18n;

const ITEM: &str = "out";
const PREFIX: &str = "bins:";
/// L'heure du rappel, le soir de la veille, à l'heure du logement.
const EVENING_HOUR: u32 = 20;
/// Une case cochée se garde deux mois : au-delà, la frise ne remonte plus jusqu'à elle.
const DONE_TTL_SECONDS: u32 = 60 * 24 * 3600;

#[portaki_sdk::query(
    name = "timelineTasks",
    example(
        label = "Semaine avec une collecte",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","from":"2026-06-15T00:00:00Z","to":"2026-06-22T00:00:00Z","stays":[]}"#
    )
)]
pub fn timeline_tasks(ctx: Context, args: TimelineTasksArgs) -> Result<TimelineTasks> {
    let config = ModuleConfig::load(&ctx)?;
    if !config.host_reminder || !config.has_collection() {
        return Ok(TimelineTasks::default());
    }
    let tz = PropertyTz::parse(&ctx.property.timezone);
    let mut tasks = Vec::new();
    for (date, at) in evenings(&config.collection_days(), args.from, args.to, tz.as_ref()) {
        tasks.push(task(args.property_id, date, at)?);
    }
    Ok(TimelineTasks { tasks })
}

/// Les collectes dont la veille au soir tombe dans `[from, to]`, avec l'heure de ce soir-là.
pub fn evenings(
    days: &[String],
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    tz: Option<&PropertyTz>,
) -> Vec<(NaiveDate, DateTime<Utc>)> {
    let wanted: Vec<_> = days.iter().filter_map(|day| parse_day(day)).collect();
    if wanted.is_empty() || to < from {
        return Vec::new();
    }
    let local = |at: DateTime<Utc>| match tz {
        Some(tz) => tz.to_local(at).date_naive(),
        None => at.date_naive(),
    };
    let (first, last) = (local(from), local(to));
    first
        .iter_days()
        .take_while(|eve| *eve <= last)
        .filter_map(|eve| {
            let collection = eve.checked_add_days(Days::new(1))?;
            if !wanted.contains(&collection.weekday()) {
                return None;
            }
            let evening = eve.and_hms_opt(EVENING_HOUR, 0, 0)?;
            let at = match tz {
                Some(tz) => tz.from_local(evening),
                None => evening.and_utc(),
            };
            (from..=to).contains(&at).then_some((collection, at))
        })
        .collect()
}

fn task(property_id: Uuid, collection: NaiveDate, at: DateTime<Utc>) -> Result<TimelineTask> {
    let id = task_id(collection);
    let mut item = TimelineTaskItem::new(ITEM, i18n::text("task.item"));
    item.done = kv::get(&kv_key(&id))?.is_some();
    Ok(timeline::task(
        id,
        at,
        property_id,
        i18n::text("task.title"),
        i18n::text("task.context"),
    )
    .due_at(at)
    .items(vec![item]))
}

pub fn task_id(collection: NaiveDate) -> String {
    format!("{PREFIX}{collection}")
}

/// La date de collecte d'un id de tâche.
fn parse_task_id(task_id: &str) -> Option<NaiveDate> {
    task_id.strip_prefix(PREFIX)?.parse().ok()
}

fn kv_key(task_id: &str) -> String {
    format!("task:{task_id}")
}

#[portaki_sdk::command(
    name = "taskToggle",
    example(
        label = "Bacs sortis",
        input = r#"{"propertyId":"5a1c9e2b-4f3d-4b8a-8e7f-6d2c1b0a9f8e","taskId":"bins:2026-06-16","itemId":"out","done":true}"#
    )
)]
pub fn task_toggle(ctx: Context, args: TaskToggleArgs) -> Result<()> {
    if args.item_id != ITEM {
        return Err(PortakiError::Host("item_not_found".to_string()));
    }
    set_done(&ctx, &args.task_id, args.done)
}

#[portaki_sdk::command(
    name = "taskComplete",
    example(
        label = "Bacs sortis",
        input = r#"{"propertyId":"5a1c9e2b-4f3d-4b8a-8e7f-6d2c1b0a9f8e","taskId":"bins:2026-06-16"}"#
    )
)]
pub fn task_complete(ctx: Context, args: TaskCompleteArgs) -> Result<()> {
    set_done(&ctx, &args.task_id, true)
}

/// Coche ou décoche « Bacs sortis » — décocher se permet : l'hôte s'est trompé de soir.
fn set_done(ctx: &Context, id: &str, done: bool) -> Result<()> {
    if ctx.guest.is_some() {
        return Err(PortakiError::Host("host_only".to_string()));
    }
    let collection =
        parse_task_id(id).ok_or_else(|| PortakiError::Host("task_not_found".to_string()))?;
    let key = kv_key(&task_id(collection));
    if done {
        kv::set(&key, b"1", Some(DONE_TTL_SECONDS))
    } else {
        kv::delete(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(raw: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(raw)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn days(list: &[&str]) -> Vec<String> {
        list.iter().map(|d| d.to_string()).collect()
    }

    /// Collecte le mardi : la veille, lundi à 20 h heure de Paris (18 h UTC en été).
    #[test]
    fn the_task_is_the_evening_before_each_collection() {
        let tz = PropertyTz::parse("Europe/Paris");
        let found = evenings(
            &days(&["tue"]),
            at("2026-10-05T00:00:00Z"),
            at("2026-10-19T00:00:00Z"),
            tz.as_ref(),
        );
        assert_eq!(
            found,
            [
                (
                    NaiveDate::from_ymd_opt(2026, 10, 6).unwrap(),
                    at("2026-10-05T18:00:00Z")
                ),
                (
                    NaiveDate::from_ymd_opt(2026, 10, 13).unwrap(),
                    at("2026-10-12T18:00:00Z")
                ),
            ]
        );
    }

    /// Une veille hors de la fenêtre n'y entre pas, et sans jour coché il n'y a rien.
    #[test]
    fn only_evenings_inside_the_window() {
        let found = evenings(
            &days(&["tue"]),
            at("2026-10-05T21:00:00Z"),
            at("2026-10-12T19:00:00Z"),
            None,
        );
        assert!(found.is_empty());
        assert!(evenings(
            &[],
            at("2026-10-05T00:00:00Z"),
            at("2026-10-19T00:00:00Z"),
            None
        )
        .is_empty());
    }

    #[test]
    fn the_id_carries_the_collection_date() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 13).unwrap();
        assert_eq!(task_id(date), "bins:2026-10-13");
        assert_eq!(parse_task_id("bins:2026-10-13"), Some(date));
        assert_eq!(parse_task_id("bins:demain"), None);
        assert_eq!(parse_task_id("2026-10-13"), None);
    }
}
