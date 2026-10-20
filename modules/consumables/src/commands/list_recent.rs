//! `listRecent` — les dernières demandes.

use portaki_sdk::prelude::*;

use super::list_for_stay::ConsumableReportRow;
use crate::storage;

#[portaki_sdk::query(name = "listRecent", example(label = "Derniers signalements"))]
pub fn list_recent(_ctx: Context) -> Result<Vec<ConsumableReportRow>> {
    Ok(storage::list_recent()?
        .into_iter()
        .map(ConsumableReportRow::from)
        .collect())
}
