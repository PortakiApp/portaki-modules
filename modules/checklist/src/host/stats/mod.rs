//! Statistics — tiles (`statsSummary`) and details for « Checklist » (guest lists) and
//! « Ménage » (host cleaning lists).
//!
//! Only what the module stores is shown: guest ticks per stay, host ticks per task. The platform
//! passes the period's stays to the detail page (`input.stays` — dates and status, no guest name):
//! with them, a stay that never opened its list shows as « Non remplie » and a cleaning is judged
//! against its deadline. Without them (developer preview, older platform), both stay unknown.

use std::collections::BTreeMap;

use chrono::{DateTime, Duration, Utc};
use portaki_sdk::contracts::stats::{self, AttentionLevel, StatsSummary, StatsSummaryArgs};
use portaki_sdk::contracts::timeline::TimelineStay;
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Chart, EmptyState, FeedItem, Grid, Page, Stack, Stat};
use portaki_sdk::sdui::surface::Surface;
use portaki_sdk::sdui::{ChartKind, ChartPoint, FeedStatus};
use uuid::Uuid;

use crate::entities::TaskItemState;
use crate::storage;
use crate::tasks::{parse_task_id, plan_tasks, task_id};
use crate::{lists, List, Step};

#[portaki_sdk::query(
    name = "statsSummary",
    example(
        label = "Checklists sur 30 jours",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","period":30,"key":"checklist"}"#
    ),
    example(
        label = "Ménage sur 90 jours",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","period":90,"key":"cleaning"}"#
    )
)]
pub fn stats_summary(_ctx: Context, args: StatsSummaryArgs) -> Result<StatsSummary> {
    let since = time::now()? - Duration::days(i64::from(args.period));
    match args.key.as_str() {
        "checklist" => {
            let stays = guest_stays(since)?;
            let value = percent(complete_count(&stays), stays.len());
            Ok(stats::summary(
                value,
                portaki_sdk::bundle_text!("stats.checklist.tile", &[]),
            ))
        }
        "cleaning" => {
            let data = cleaning_data()?;
            let tasks = cleaning_tasks_of(&data, since);
            let open = tasks.iter().filter(|task| !task.finished()).count();
            let finished = tasks.len() - open;
            let tile = stats::summary(
                percent(finished, tasks.len()),
                portaki_sdk::bundle_text!("stats.cleaning.tile", &[]),
            );
            if open == 0 {
                return Ok(tile);
            }
            let count = open.to_string();
            Ok(tile.attention(
                AttentionLevel::Action,
                portaki_sdk::bundle_text!("stats.cleaning.attention", &[("count", &count)]),
            ))
        }
        other => Err(PortakiError::Host(format!("unknown_stats_key:{other}"))),
    }
}

fn percent(part: usize, whole: usize) -> String {
    match (part * 100).checked_div(whole) {
        Some(value) => format!("{value} %"),
        None => "—".to_string(),
    }
}

/// Dashboard period selector: 30 j / 90 j / 12 mois; anything else falls back to 30.
fn period_days(ctx: &HostContext) -> i64 {
    match ctx.input_u64("periodDays") {
        Some(90) => 90,
        Some(365) => 365,
        _ => 30,
    }
}

/// The period's stays the platform passes to the detail page, oldest first; `None` when it sends
/// none (developer preview, older platform).
fn period_stays(ctx: &HostContext) -> Option<Vec<TimelineStay>> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Raw {
        id: Uuid,
        check_in: DateTime<Utc>,
        check_out: DateTime<Utc>,
        #[serde(default)]
        status: String,
    }
    let raw: Vec<Raw> = serde_json::from_value(ctx.input.get("stays")?.clone()).ok()?;
    let mut stays: Vec<TimelineStay> = raw
        .into_iter()
        .map(|stay| TimelineStay {
            id: stay.id,
            check_in: stay.check_in,
            check_out: stay.check_out,
            guest_name: String::new(),
            status: stay.status,
        })
        .collect();
    stays.sort_by_key(|stay| stay.check_in);
    Some(stays)
}

mod cleaning;
mod cleaning_tasks;
mod guest;
mod shared;

pub use cleaning::render_stats_cleaning;
use cleaning_tasks::*;
pub use guest::render_stats_checklist;
use guest::*;
use shared::*;

fn is_fr(locale: &str) -> bool {
    locale.trim().to_ascii_lowercase().starts_with("fr")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats() {
        assert_eq!(format_duration(Duration::minutes(130)), "2 h 10");
        assert_eq!(format_duration(Duration::minutes(40)), "40 min");
        assert_eq!(format_duration(Duration::minutes(120)), "2 h");
        assert_eq!(decimal(5.14, true), "5,1");
        assert_eq!(percent(7, 9), "77 %");
        assert_eq!(percent(0, 0), "—");
    }
}
