//! La tuile « Modes d'emploi ouverts » : le compte des fiches ouvertes sur la période.

use portaki_sdk::contracts::stats::{self, StatsSummary, StatsSummaryArgs};
use portaki_sdk::prelude::*;

use crate::host::appliances_views::ITEM;

#[portaki_sdk::query(
    name = "statsSummary",
    example(
        label = "Modes d'emploi sur 30 jours",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","period":30,"key":"appliances-views","views":{"opens":9,"surfaces":{"explore.item":7,"explore.detail":2},"clicks":{"link":3}}}"#
    )
)]
pub fn stats_summary(_ctx: Context, args: StatsSummaryArgs) -> Result<StatsSummary> {
    let value = args
        .views
        .map_or_else(|| "—".to_owned(), |views| views.surface(ITEM).to_string());
    Ok(stats::summary(
        value,
        portaki_sdk::bundle_text!("stats.tile"),
    ))
}
