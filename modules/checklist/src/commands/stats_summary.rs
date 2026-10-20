//! `statsSummary`: the tiles of « Checklist » (guest lists) and « Ménage » (host cleaning lists).

use chrono::Duration;
use portaki_sdk::contracts::stats::{self, AttentionLevel, StatsSummary, StatsSummaryArgs};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;

use crate::host::stats::{cleaning_data, cleaning_tasks_of, complete_count, guest_stays, percent};

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
