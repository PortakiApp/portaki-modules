//! `taskToggle` — cocher ou décocher « Marquer livré ».

use portaki_sdk::contracts::timeline::TaskToggleArgs;
use portaki_sdk::prelude::*;

use super::tasks::{set_status, ITEM};
use crate::status;

#[portaki_sdk::command(
    name = "taskToggle",
    example(
        label = "Marquer livré",
        input = r#"{"propertyId":"5a1c9e2b-4f3d-4b8a-8e7f-6d2c1b0a9f8e","taskId":"9d8c7b6a-5f4e-4d3c-8b2a-1f0e9d8c7b6a","itemId":"restocked","done":true}"#
    )
)]
pub fn task_toggle(_ctx: Context, args: TaskToggleArgs) -> Result<()> {
    if args.item_id != ITEM {
        return Err(PortakiError::Host("item_not_found".to_string()));
    }
    // Décocher rouvre la demande : l'hôte s'est trompé de ligne.
    let next = if args.done {
        status::RESTOCKED
    } else {
        status::DEFAULT
    };
    set_status(&args.task_id, next)
}
