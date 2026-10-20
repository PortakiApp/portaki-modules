//! Property statistics « Consommables » — the tile (`statsSummary`) and the
//! `property-stats-detail` surface (`stock`).
//!
//! The module knows the catalog and the guests' shortage reports, not quantities: « À racheter »
//! is an item with an open report, « Rupture » one reported empty. With no quantity, « how much per
//! stay » becomes « how often it ran out per stay » — from the period's stays the platform passes
//! (`input.stays`) — and « when to buy again » the delay between a restock and the next shortage.

use chrono::{DateTime, Duration, Utc};
use portaki_sdk::contracts::stats::{self, AttentionLevel, StatsSummary, StatsSummaryArgs};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, EmptyState, Grid, Page, Stack, Stat};
use portaki_sdk::sdui::surface::Surface;

use uuid::Uuid;

use crate::entities::ConsumableReport;
use crate::{status, storage, Product, Products};

/// Where an item of the catalog stands.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stock {
    Out,
    Low,
    Ok,
}

/// The item's open reports decide: one « missing » is a stock-out.
fn stock(item: &Product, reports: &[ConsumableReport]) -> Stock {
    let open: Vec<&ConsumableReport> = reports
        .iter()
        .filter(|r| r.item_id == item.uuid() && status::is_pending(&r.status))
        .collect();
    if open.iter().any(|r| r.level == "missing") {
        Stock::Out
    } else if open.is_empty() {
        Stock::Ok
    } else {
        Stock::Low
    }
}

/// Departures of `(since, now]` among the stays the platform passed; `None` when it passed none.
fn departures(ctx: &HostContext, since: DateTime<Utc>, now: DateTime<Utc>) -> Option<Vec<Uuid>> {
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
fn days_between_restocks(item: &Product, reports: &[ConsumableReport]) -> Option<i64> {
    let mine: Vec<&ConsumableReport> = reports
        .iter()
        .filter(|r| r.item_id == item.uuid())
        .collect();
    let gaps: Vec<i64> = mine
        .iter()
        .filter_map(|r| r.restocked_at)
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

fn label(item: &Product, locale: &str) -> String {
    item.name(locale)
}

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

/// Suivis, à racheter, demandés sur la période, dernier réassort.
fn tiles(
    items: &[Product],
    reports: &[ConsumableReport],
    asked: usize,
    days: i64,
    fr: bool,
) -> Grid {
    let to_buy = items
        .iter()
        .filter(|item| stock(item, reports) != Stock::Ok)
        .count();
    let last_restock = reports.iter().filter_map(|r| r.restocked_at).max();
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

#[portaki_sdk::surface(
    host,
    id = "stock",
    placement = HostPlacement::PropertyStatsCard,
    placement = HostPlacement::PropertyStatsDetail,
    label_key = "catalog.host.stock",
    icon = IconName::Package
)]
pub fn render_host_stats(ctx: HostContext) -> Surface {
    let locale = ctx.locale.as_str();
    let fr = locale.to_ascii_lowercase().starts_with("fr");
    let days: i64 = match ctx.input_u64("periodDays") {
        Some(90) => 90,
        Some(365) => 365,
        _ => 30,
    };
    let now = time::now().unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    let items = Products::load(&ctx)
        .map(|products| products.rows().to_vec())
        .unwrap_or_default();
    let reports = storage::list_all().unwrap_or_default();
    let period: Vec<&ConsumableReport> = reports
        .iter()
        .filter(|r| r.created_at >= now - Duration::days(days))
        .collect();

    if items.is_empty() {
        return Surface::new(
            Page::new().child(
                EmptyState::new()
                    .title("i18n:stats.empty")
                    .description("i18n:stats.emptyHint")
                    .icon(IconName::Package),
            ),
        )
        .with_id(STOCK);
    }

    let tiles = tiles(&items, &reports, period.len(), days, fr);

    let chart = match departures(&ctx, now - Duration::days(days), now) {
        Some(stays) if !stays.is_empty() => per_stay_chart(&items, &period, &stays, locale),
        _ => reports_chart(&items, &period, locale),
    };
    let restock = restock_chart(&items, &reports, locale);

    let config = Action::navigate(
        NavigateTarget::path(format!("/listings/{}/modules/consumables", ctx.property_id)),
        None,
    );
    let rows = items
        .iter()
        .map(|item| stock_row(item, &reports, locale, fr, config.clone()))
        .collect();

    Surface::new(Page::new().child(Stack::new().gap(16.0).children(vec![
        tiles.into(),
        Grid::new()
            .minColumnWidth(320.0)
            .gap(16.0)
            .children(vec![chart, restock])
            .into(),
        Card::new()
            .title("i18n:stats.feed.title")
            .subtitle("i18n:stats.feed.subtitle")
            .child(Stack::new().gap(0.0).children(rows))
            .into(),
    ])))
    .with_id(STOCK)
}

mod charts;

use charts::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn report(item: Uuid, created: i64, restocked: Option<i64>) -> ConsumableReport {
        let day = |d: i64| DateTime::<Utc>::UNIX_EPOCH + Duration::days(d);
        ConsumableReport {
            id: Uuid::new_v4(),
            stay_id: Uuid::new_v4(),
            item_id: item,
            item_label: String::new(),
            level: "missing".into(),
            note: None,
            status: status::DEFAULT.into(),
            created_at: day(created),
            restocked_at: restocked.map(day),
            host_reply: None,
            guest_name: String::new(),
            stay_check_in: None,
            stay_check_out: None,
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
