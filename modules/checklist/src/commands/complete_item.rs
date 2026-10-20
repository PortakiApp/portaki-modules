//! `completeItem` : le voyageur coche une étape.

use portaki_sdk::prelude::*;
use uuid::Uuid;

use super::progress::{emit_progress, require_guest_item, require_stay_id};
use crate::storage;

/// Arguments for complete / uncomplete.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct ItemIdArgs {
    pub item_id: Uuid,
}

#[portaki_sdk::command(
    name = "completeItem",
    guest,
    example(
        label = "Cocher une tâche",
        input = r#"{"itemId":"8c0e5a4b-1d2f-4e6a-9b7c-3f1d2e4a5b6c"}"#
    )
)]
pub fn complete_item(ctx: Context, args: ItemIdArgs) -> Result<()> {
    let stay_id = require_stay_id(&ctx)?;
    require_guest_item(args.item_id)?;
    storage::complete_item(stay_id, args.item_id)?;
    emit_progress(ctx.property_id, stay_id)
}
