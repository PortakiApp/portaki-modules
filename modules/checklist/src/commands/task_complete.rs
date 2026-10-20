//! `taskComplete`: every open item of a host task ticked at once.

use portaki_sdk::contracts::timeline::TaskCompleteArgs;
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;

use super::task::{after_change, is_restock, load_task, state_of, task_item};
use crate::{storage, Step};

#[portaki_sdk::command(
    name = "taskComplete",
    example(
        label = "Terminer le ménage",
        input = r#"{"propertyId":"5a1c9e2b-4f3d-4b8a-8e7f-6d2c1b0a9f8e","taskId":"3f2b8c1e-7a4d-4e9b-9c61-2d5e8f0a1b47:7d4e2f1a-9b8c-4a3d-8e6f-5c4b3a2d1e0f"}"#
    )
)]
pub fn task_complete(ctx: Context, args: TaskCompleteArgs) -> Result<()> {
    let (list, items, stay_id) = load_task(&ctx, &args.task_id)?;
    let states = storage::task_states(Some(&args.task_id))?;
    let open: Vec<&Step> = items
        .iter()
        .filter(|item| !state_of(&states, &args.task_id, item.uuid()).is_some_and(|s| s.done))
        .collect();
    // Refuse before ticking anything: a half-completed task would be worse than none.
    for item in &open {
        let stored = state_of(&states, &args.task_id, item.uuid()).and_then(|s| s.photo.as_deref());
        task_item(&list, item, None).check_toggle(true, stored)?;
    }
    let now = time::now()?;
    for item in &open {
        storage::set_task_item(&args.task_id, item.uuid(), true, None, now)?;
    }
    let restocked = open.iter().any(|item| is_restock(item));
    after_change(&ctx, &list, &args.task_id, stay_id, restocked)
}
