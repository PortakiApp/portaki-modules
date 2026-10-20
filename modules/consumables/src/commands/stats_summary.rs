//! `statsSummary` — the « Consommables » tile of the property statistics.

use portaki_sdk::contracts::stats::{self, AttentionLevel, StatsSummary, StatsSummaryArgs};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::ConsumableReport;
use crate::{storage, Products};

/// Tile (spec §1) : « Demandes de réassort · 6 » over the period, « Café en tête » as its note.
#[portaki_sdk::query(
    name = "statsSummary",
    example(
        label = "Stock sur 30 jours",
        input = r#"{"propertyId":"8c0e6f2a-1d3b-4a7e-b5c9-0f4e2d1a6b38","period":30,"key":"stock"}"#
    )
)]
pub fn stats_summary(ctx: Context, args: StatsSummaryArgs) -> Result<StatsSummary> {
    let since = args.period().since(time::now()?);
    let period: Vec<ConsumableReport> = storage::list_all()?
        .into_iter()
        .filter(|r| r.created_at >= since)
        .collect();
    let tile = stats::summary(
        period.len().to_string(),
        portaki_sdk::bundle_text!("stats.tile", &[]),
    );
    let Some(top) = top_item(&period) else {
        return Ok(tile);
    };
    // One text per language: the item is named in each — the catalog's names, or the snapshot
    // the guest saw when the item has left the catalog since.
    let products = Products::load(&ctx)?;
    let (fr, en) = match products.rows().iter().find(|p| p.uuid() == top.item_id) {
        Some(item) => (item.name("fr"), item.name("en")),
        None => (top.item_label.clone(), top.item_label.clone()),
    };
    let mut text = portaki_sdk::bundle_text!("stats.tile.top", &[]);
    text.fr = text.fr.replace("{item}", &fr);
    text.en = text.en.replace("{item}", &en);
    Ok(tile.attention(AttentionLevel::Warning, text))
}

/// The most requested item of the period (the most recent request breaks a tie).
fn top_item(period: &[ConsumableReport]) -> Option<&ConsumableReport> {
    let count = |id: Uuid| period.iter().filter(|r| r.item_id == id).count();
    period.iter().max_by(|a, b| {
        count(a.item_id)
            .cmp(&count(b.item_id))
            .then(a.created_at.cmp(&b.created_at))
    })
}
