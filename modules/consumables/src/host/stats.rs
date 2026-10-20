//! Property statistics « Consommables » — the tile (`statsSummary`) and the
//! `property-stats-detail` surface (`stock`).
//!
//! The module knows the catalog and the guests' shortage reports, not quantities: « À racheter »
//! is an item with an open report, « Rupture » one reported empty. With no quantity, « how much per
//! stay » becomes « how often it ran out per stay » — from the period's stays the platform passes
//! (`input.stays`) — and « when to buy again » the delay between a restock and the next shortage.

use chrono::{DateTime, Utc};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Grid, Stat};
use uuid::Uuid;

use super::charts::short_date;
use crate::domain::request::Asked;
use crate::{Entry, Level, Product};

/// Where an item of the catalog stands.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Stock {
    Out,
    Low,
    Ok,
}

/// The item's open reports decide: one « missing » is a stock-out.
pub(super) fn stock(item: &Product, reports: &[Entry]) -> Stock {
    let open: Vec<&Entry> = reports
        .iter()
        .filter(|r| r.product() == item.uuid() && r.pending())
        .collect();
    if open.iter().any(|r| r.item.level == Level::Missing) {
        Stock::Out
    } else if open.is_empty() {
        Stock::Ok
    } else {
        Stock::Low
    }
}

/// Departures of `(since, now]` among the stays the platform passed; `None` when it passed none.
pub(super) fn departures(
    ctx: &HostContext,
    since: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Option<Vec<Uuid>> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Stay {
        id: Uuid,
        check_out: DateTime<Utc>,
    }
    let stays: Vec<Stay> = serde_json::from_value(ctx.input.get("stays")?.clone()).ok()?;
    Some(
        stays
            .into_iter()
            .filter(|stay| stay.check_out > since && stay.check_out <= now)
            .map(|stay| stay.id)
            .collect(),
    )
}

/// Mean days between a restock of `item` and its next shortage report, over all its history.
pub(super) fn days_between_restocks(item: &Product, reports: &[Entry]) -> Option<i64> {
    let mine: Vec<&Entry> = reports
        .iter()
        .filter(|r| r.product() == item.uuid())
        .collect();
    let gaps: Vec<i64> = mine
        .iter()
        .filter_map(|r| r.restocked_at())
        .filter_map(|restocked| {
            let next = mine
                .iter()
                .map(|r| r.created_at)
                .filter(|at| *at > restocked)
                .min()?;
            Some((next - restocked).num_days())
        })
        .collect();
    let n = i64::try_from(gaps.len()).ok().filter(|n| *n > 0)?;
    Some(gaps.iter().sum::<i64>() / n)
}

pub(super) fn label(item: &Product, locale: &str) -> String {
    item.name(locale)
}

/// Suivis, à racheter, demandés sur la période, dernier réassort.
pub(super) fn tiles(
    items: &[Product],
    reports: &[Entry],
    asked: usize,
    days: i64,
    fr: bool,
) -> Grid {
    let to_buy = items
        .iter()
        .filter(|item| stock(item, reports) != Stock::Ok)
        .count();
    let last_restock = reports.iter().filter_map(|r| r.restocked_at()).max();
    Grid::new().minColumnWidth(170.0).gap(12.0).children(vec![
        Stat::new()
            .label("i18n:stats.tracked")
            .icon(IconName::Package)
            .value(items.len().to_string())
            .into(),
        Stat::new()
            .label("i18n:stats.toBuy")
            .icon(IconName::DangerTriangle)
            .value(to_buy.to_string())
            .delta("i18n:stats.toBuy.note")
            .into(),
        Stat::new()
            .label("i18n:stats.reported")
            .icon(IconName::Message)
            .value(asked.to_string())
            .delta(t!(&format!("stats.window.{days}")).unwrap_or_default())
            .into(),
        Stat::new()
            .label("i18n:stats.lastRestock")
            .icon(IconName::Calendar)
            .value(last_restock.map_or_else(|| "—".to_string(), |at| short_date(at, fr)))
            .into(),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use portaki_sdk::resource::{Actor, HistoryEvent, HistoryLine};

    fn report(item: Uuid, created: i64, restocked: Option<i64>) -> Entry {
        let day = |d: i64| DateTime::<Utc>::UNIX_EPOCH + Duration::days(d);
        let history = restocked
            .map(|d| HistoryLine {
                at: day(d),
                actor: Actor::Host,
                event: HistoryEvent::Status {
                    to: "restocked".into(),
                },
            })
            .into_iter()
            .collect();
        Entry {
            id: Uuid::new_v4(),
            item: crate::Request {
                item_id: item.to_string(),
                ..Default::default()
            },
            status: restocked.map_or(crate::Status::Open, |_| crate::Status::Restocked),
            stay_id: Some(Uuid::new_v4()),
            created_at: day(created),
            actor: Actor::Guest,
            guest_name: String::new(),
            check_in: None,
            check_out: None,
            history,
        }
    }

    /// Réassort au jour 2, manque au jour 12 ; réassort au jour 13, manque au jour 19 : 8 jours.
    #[test]
    fn restock_rhythm_is_the_mean_gap_to_the_next_shortage() {
        let item = Product {
            id: Uuid::new_v4().to_string(),
            ..Product::default()
        };
        let other = Uuid::new_v4();
        let reports = vec![
            report(item.uuid(), 0, Some(2)),
            report(item.uuid(), 12, Some(13)),
            report(item.uuid(), 19, None),
            report(other, 5, None),
        ];
        assert_eq!(days_between_restocks(&item, &reports), Some(8));
        assert_eq!(days_between_restocks(&item, &reports[2..]), None);
    }
}
