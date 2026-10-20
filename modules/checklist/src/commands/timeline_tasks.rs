//! `timelineTasks`: host lists as dated tasks.

use portaki_sdk::contracts::timeline::{TimelineTasks, TimelineTasksArgs};
use portaki_sdk::prelude::*;

use super::task::{build_task, plan_tasks};
use crate::{storage, List};

#[portaki_sdk::query(
    name = "timelineTasks",
    example(
        label = "Semaine avec un départ",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","from":"2026-06-15T00:00:00Z","to":"2026-06-22T00:00:00Z","stays":[{"id":"8d3e6f2a-1b4c-4d5e-9f60-7a8b9c0d1e2f","checkIn":"2026-06-12T14:00:00Z","checkOut":"2026-06-17T09:00:00Z","guestName":"Camille Durand","status":"ACTIVE"},{"id":"2c4e6a8b-0d1f-4a3c-8e5b-7d9f1a3c5e70","checkIn":"2026-06-19T14:00:00Z","checkOut":"2026-06-26T09:00:00Z","guestName":"Lucas Martin","status":"UPCOMING"}]}"#
    )
)]
pub fn timeline_tasks(ctx: Context, args: TimelineTasksArgs) -> Result<TimelineTasks> {
    let lists: Vec<List> = crate::domain::all_lists()?
        .into_iter()
        .filter(|list| !list.is_guest())
        .collect();
    if lists.is_empty() {
        return Ok(TimelineTasks::default());
    }
    let states = storage::task_states(None)?;
    let mut tasks = Vec::new();
    for list in &lists {
        let list_items = list.written_steps();
        if list_items.is_empty() {
            continue;
        }
        for plan in plan_tasks(list, &args.stays, &ctx.property.timezone) {
            if plan.at < args.from || plan.at > args.to {
                continue;
            }
            tasks.push(build_task(
                args.property_id,
                list,
                &list_items,
                &states,
                plan,
            ));
        }
    }
    tasks.sort_by_key(|task| task.at);
    Ok(TimelineTasks { tasks })
}
