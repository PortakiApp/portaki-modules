//! `listCompletions` : les coches du séjour.

use portaki_sdk::prelude::*;
use uuid::Uuid;

#[portaki_sdk::query(name = "listCompletions", example(label = "Tâches cochées du séjour"))]
pub fn list_completions(ctx: Context) -> Result<Vec<Uuid>> {
    let stay_id = ctx
        .guest
        .as_ref()
        .map(|guest| guest.session_id)
        .ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))?;
    Ok(crate::storage::list_completions(Some(stay_id))?
        .into_iter()
        .map(|row| row.item_id)
        .collect())
}
