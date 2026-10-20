//! `taskComplete` — « Marquer livré ».

use portaki_sdk::contracts::timeline::TaskCompleteArgs;
use portaki_sdk::prelude::*;

use super::tasks::set_status;
use crate::status;

#[portaki_sdk::command(
    name = "taskComplete",
    example(
        label = "Marquer livré",
        input = r#"{"propertyId":"5a1c9e2b-4f3d-4b8a-8e7f-6d2c1b0a9f8e","taskId":"9d8c7b6a-5f4e-4d3c-8b2a-1f0e9d8c7b6a"}"#
    )
)]
pub fn task_complete(_ctx: Context, args: TaskCompleteArgs) -> Result<()> {
    set_status(&args.task_id, status::RESTOCKED)
}
