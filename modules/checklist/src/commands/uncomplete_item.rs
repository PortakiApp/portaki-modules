//! `uncompleteItem` : le voyageur décoche une étape.

use portaki_sdk::prelude::*;

use super::complete_item::ItemIdArgs;
use super::progress::{emit_progress, require_guest_item, require_stay_id};
use crate::storage;

#[portaki_sdk::command(
    name = "uncompleteItem",
    guest,
    example(
        label = "Décocher une tâche",
        input = r#"{"itemId":"8c0e5a4b-1d2f-4e6a-9b7c-3f1d2e4a5b6c"}"#
    )
)]
pub fn uncomplete_item(ctx: Context, args: ItemIdArgs) -> Result<()> {
    let stay_id = require_stay_id(&ctx)?;
    require_guest_item(args.item_id)?;
    storage::uncomplete_item(stay_id, args.item_id)?;
    emit_progress(ctx.property_id, stay_id)
}
