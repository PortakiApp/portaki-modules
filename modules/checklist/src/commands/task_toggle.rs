//! `taskToggle`: one item of a host task ticked or unticked.

use portaki_sdk::contracts::timeline::TaskToggleArgs;
use portaki_sdk::files::FileRef;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use super::task::{after_change, is_restock, load_task, state_of, task_item};
use crate::storage;

#[portaki_sdk::command(
    name = "taskToggle",
    example(
        label = "Cocher une tâche du ménage",
        input = r#"{"propertyId":"5a1c9e2b-4f3d-4b8a-8e7f-6d2c1b0a9f8e","taskId":"3f2b8c1e-7a4d-4e9b-9c61-2d5e8f0a1b47:7d4e2f1a-9b8c-4a3d-8e6f-5c4b3a2d1e0f","itemId":"8c0e5a4b-1d2f-4e6a-9b7c-3f1d2e4a5b6c","done":true}"#
    )
)]
pub fn task_toggle(ctx: Context, args: TaskToggleArgs) -> Result<()> {
    let (list, items, stay_id) = load_task(&ctx, &args.task_id)?;
    let item = Uuid::parse_str(&args.item_id)
        .ok()
        .and_then(|id| items.iter().find(|item| item.uuid() == id))
        .ok_or_else(|| PortakiError::Host("item_not_found".to_string()))?;
    let photo = match args
        .photo
        .as_deref()
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        None => None,
        Some(raw) => Some(
            FileRef::parse(raw)
                .ok_or_else(|| PortakiError::Host("invalid_photo".to_string()))?
                .to_string(),
        ),
    };
    let states = storage::task_states(Some(&args.task_id))?;
    let stored = state_of(&states, &args.task_id, item.uuid()).and_then(|s| s.photo.clone());
    task_item(&list, item, None).check_toggle(args.done, photo.as_deref().or(stored.as_deref()))?;

    storage::set_task_item(&args.task_id, item.uuid(), args.done, photo)?;
    let restocked = args.done && is_restock(item);
    after_change(&ctx, &list, &args.task_id, stay_id, restocked)
}
