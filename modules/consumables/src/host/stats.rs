//! Property statistics « Consommables » — the tile (`statsSummary`) and the
//! `property-stats-detail` surface (`stock`).
//!
//! The module knows the catalog and the guests' shortage reports, not quantities: « À racheter »
//! is an item with an open report, « Rupture » one reported empty. With no quantity, « how much per
//! stay » becomes « how often it ran out per stay » — from the period's stays the platform passes
//! (`input.stays`) — and « when to buy again » the delay between a restock and the next shortage.

use chrono::{DateTime, Datelike, Duration, Utc};
use portaki_sdk::contracts::stats::{self, AttentionLevel, StatsSummary, StatsSummaryArgs};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Chart, EmptyState, FeedItem, Grid, Page, Stack, Stat};
use portaki_sdk::sdui::surface::Surface;
use portaki_sdk::sdui::{ChartKind, ChartPoint, FeedStatus};

use uuid::Uuid;

use crate::entities::{ConsumableItem, ConsumableReport};
use crate::labels::{labels_from_item, pick_label};
use crate::{i18n, status, storage};

/// Where an item of the catalog stands.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stock {
    Out,
    Low,
    Ok,
}

/// The item's open reports decide: one « missing » is a stock-out.
fn stock(item: &ConsumableItem, reports: &[ConsumableReport]) -> Stock {
    let open: Vec<&ConsumableReport> = reports
        .iter()
        .filter(|r| r.item_id == item.id && r.status == status::DEFAULT)
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
fn days_between_restocks(item: &ConsumableItem, reports: &[ConsumableReport]) -> Option<i64> {
    let mine: Vec<&ConsumableReport> = reports.iter().filter(|r| r.item_id == item.id).collect();
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

fn label(item: &ConsumableItem, locale: &str) -> String {
    pick_label(&labels_from_item(item), locale, "fr")
}

/// Tile: items to buy again; a stock-out as attention.
#[portaki_sdk::query(name = "statsSummary")]
pub fn stats_summary(_ctx: Context, _args: StatsSummaryArgs) -> Result<StatsSummary> {
    let items = storage::list_items()?;
    let reports = storage::list_all()?;
    let to_buy = items
        .iter()
        .filter(|item| stock(item, &reports) != Stock::Ok)
        .count();
    let tile = stats::summary(to_buy.to_string(), i18n::text("stats.tile", &[]));
    let out: Vec<&ConsumableItem> = items
        .iter()
        .filter(|item| stock(item, &reports) == Stock::Out)
        .collect();
    let text = match out.as_slice() {
        [] => return Ok(tile),
        [item] => {
            // One text per language: the item is named in each.
            let labels = labels_from_item(item);
            let mut text = i18n::text("stats.tile.out", &[]);
            text.fr = text.fr.replace("{item}", &pick_label(&labels, "fr", "en"));
            text.en = text.en.replace("{item}", &pick_label(&labels, "en", "fr"));
            text
        }
        many => i18n::text("stats.tile.outMany", &[("count", &many.len().to_string())]),
    };
    Ok(tile.attention(AttentionLevel::Action, text))
}

#[portaki_sdk::surface(host, id = "stock")]
pub fn render_host_stats(ctx: HostContext) -> Surface {
    let locale = ctx.locale.as_str();
    let fr = locale.to_ascii_lowercase().starts_with("fr");
    let days: i64 = match ctx.input_u64("periodDays") {
        Some(90) => 90,
        Some(365) => 365,
        _ => 30,
    };
    let now = time::now().unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    let items = storage::list_items().unwrap_or_default();
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
                    .icon("package"),
            ),
        )
        .with_id(crate::ids::HOST_STATS);
    }

    let to_buy = items
        .iter()
        .filter(|item| stock(item, &reports) != Stock::Ok)
        .count();
    let last_restock = reports.iter().filter_map(|r| r.restocked_at).max();
    let tiles = Grid::new().minColumnWidth(170.0).gap(12.0).children(vec![
        Stat::new()
            .label("i18n:stats.tracked")
            .value(items.len().to_string())
            .into(),
        Stat::new()
            .label("i18n:stats.toBuy")
            .value(to_buy.to_string())
            .delta("i18n:stats.toBuy.note")
            .into(),
        Stat::new()
            .label("i18n:stats.reported")
            .value(period.len().to_string())
            .delta(t!(&format!("stats.window.{days}")).unwrap_or_default())
            .into(),
        Stat::new()
            .label("i18n:stats.lastRestock")
            .value(last_restock.map_or_else(|| "—".to_string(), |at| short_date(at, fr)))
            .into(),
    ]);

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
    .with_id(crate::ids::HOST_STATS)
}

fn panel(key: &str, body: Component) -> Component {
    Card::new()
        .title(format!("i18n:stats.{key}.title"))
        .subtitle(format!("i18n:stats.{key}.subtitle"))
        .child(body)
        .into()
}

fn bars(swatch: Swatch, points: Vec<ChartPoint>) -> Component {
    Chart::new()
        .kind(ChartKind::HorizontalBars)
        .swatch(swatch)
        .points(points)
        .into()
}

fn nothing(description: &str) -> Component {
    EmptyState::new()
        .title("i18n:stats.empty")
        .description(description)
        .icon("package")
        .into()
}

/// Share of the period's departures where a guest reported the item missing or low.
fn per_stay_chart(
    items: &[ConsumableItem],
    period: &[&ConsumableReport],
    stays: &[Uuid],
    locale: &str,
) -> Component {
    let mut rates: Vec<(String, usize)> = items
        .iter()
        .map(|item| {
            let mut short: Vec<Uuid> = period
                .iter()
                .filter(|r| r.item_id == item.id && stays.contains(&r.stay_id))
                .map(|r| r.stay_id)
                .collect();
            short.sort();
            short.dedup();
            (label(item, locale), short.len() * 100 / stays.len())
        })
        .filter(|(_, rate)| *rate > 0)
        .collect();
    rates.sort_by_key(|(_, rate)| std::cmp::Reverse(*rate));
    let body = if rates.is_empty() {
        nothing("i18n:stats.perStay.empty")
    } else {
        bars(
            Swatch::Black,
            rates
                .into_iter()
                .map(|(label, rate)| {
                    ChartPoint::new(label, rate as f64).display(format!("{rate} %"))
                })
                .collect(),
        )
    };
    panel("perStay", body)
}

/// Reports per item over the period, most reported first — when no stay came with the request.
fn reports_chart(
    items: &[ConsumableItem],
    period: &[&ConsumableReport],
    locale: &str,
) -> Component {
    let mut per_item: Vec<(String, usize)> = items
        .iter()
        .map(|item| {
            let n = period.iter().filter(|r| r.item_id == item.id).count();
            (label(item, locale), n)
        })
        .filter(|(_, n)| *n > 0)
        .collect();
    per_item.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    let body = if per_item.is_empty() {
        nothing("i18n:stats.byItem.empty")
    } else {
        bars(
            Swatch::Black,
            per_item
                .into_iter()
                .map(|(label, n)| {
                    ChartPoint::new(label, n as f64).display(
                        t!("stats.byItem.unit", count = n).unwrap_or_else(|_| n.to_string()),
                    )
                })
                .collect(),
        )
    };
    panel("byItem", body)
}

/// « Quand il faut racheter » : the soonest to run out first.
fn restock_chart(
    items: &[ConsumableItem],
    reports: &[ConsumableReport],
    locale: &str,
) -> Component {
    let mut delays: Vec<(String, i64)> = items
        .iter()
        .filter_map(|item| Some((label(item, locale), days_between_restocks(item, reports)?)))
        .collect();
    delays.sort_by_key(|(_, days)| *days);
    let body = if delays.is_empty() {
        nothing("i18n:stats.restock.empty")
    } else {
        bars(
            Swatch::Orange,
            delays
                .into_iter()
                .map(|(label, days)| {
                    ChartPoint::new(label, days as f64).display(
                        t!("stats.restock.unit", count = days).unwrap_or_else(|_| days.to_string()),
                    )
                })
                .collect(),
        )
    };
    panel("restock", body)
}

fn stock_row(
    item: &ConsumableItem,
    reports: &[ConsumableReport],
    locale: &str,
    fr: bool,
    action: Action,
) -> Component {
    let (label_key, tone) = match stock(item, reports) {
        Stock::Out => ("i18n:stats.stock.out", Tone::Danger),
        Stock::Low => ("i18n:stats.stock.low", Tone::Warning),
        Stock::Ok => ("i18n:stats.stock.ok", Tone::Success),
    };
    let mine = || reports.iter().filter(|r| r.item_id == item.id);
    let open = mine().find(|r| r.status == status::DEFAULT);
    let date = match (open, mine().filter_map(|r| r.restocked_at).max()) {
        (Some(report), _) => t!(
            "stats.row.reportedAt",
            date = short_date(report.created_at, fr)
        ),
        (None, Some(at)) => t!("stats.row.restockedAt", date = short_date(at, fr)),
        (None, None) => Ok(String::new()),
    }
    .unwrap_or_default();
    let mut row = FeedItem::new()
        .title(label(item, locale))
        .date(date)
        .dotTone(tone)
        .status(FeedStatus::new(label_key, tone))
        .action(action);
    if let Some(note) = open.and_then(|r| r.note.clone()) {
        row = row.meta(note);
    }
    row.into()
}

fn short_date(at: DateTime<Utc>, fr: bool) -> String {
    const FR: [&str; 12] = [
        "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.",
        "déc.",
    ];
    const EN: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let month = at.month0() as usize;
    if fr {
        format!("{} {}", at.day(), FR[month])
    } else {
        format!("{} {}", EN[month], at.day())
    }
}

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
        }
    }

    /// Réassort au jour 2, manque au jour 12 ; réassort au jour 13, manque au jour 19 : 8 jours.
    #[test]
    fn restock_rhythm_is_the_mean_gap_to_the_next_shortage() {
        let item = ConsumableItem {
            id: Uuid::new_v4(),
            label_fr: "Café".into(),
            label_en: "Coffee".into(),
            sort_order: 0,
            low_threshold: 0,
            created_at: DateTime::<Utc>::UNIX_EPOCH,
        };
        let other = Uuid::new_v4();
        let reports = vec![
            report(item.id, 0, Some(2)),
            report(item.id, 12, Some(13)),
            report(item.id, 19, None),
            report(other, 5, None),
        ];
        assert_eq!(days_between_restocks(&item, &reports), Some(8));
        assert_eq!(days_between_restocks(&item, &reports[2..]), None);
    }
}
