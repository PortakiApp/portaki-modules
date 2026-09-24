//! Property statistics « Consommables » — the tile (`statsSummary`) and the
//! `property-stats-detail` surface (`stock`).
//!
//! The module knows the catalog and the guests' shortage reports, not quantities: « À racheter »
//! is an item with an open report, « Rupture » one reported empty.

use chrono::{DateTime, Datelike, Duration, Utc};
use portaki_sdk::contracts::stats::{self, AttentionLevel, StatsSummary, StatsSummaryArgs};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Chart, EmptyState, FeedItem, Grid, Page, Stack, Stat};
use portaki_sdk::sdui::surface::Surface;
use portaki_sdk::sdui::{ChartKind, ChartPoint, FeedStatus};

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

    // Reports per item over the period, most reported first.
    let mut per_item: Vec<(String, usize)> = items
        .iter()
        .map(|item| {
            let n = period.iter().filter(|r| r.item_id == item.id).count();
            (label(item, locale), n)
        })
        .filter(|(_, n)| *n > 0)
        .collect();
    per_item.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    let chart: Component = if per_item.is_empty() {
        EmptyState::new()
            .title("i18n:stats.empty")
            .description("i18n:stats.byItem.empty")
            .icon("package")
            .into()
    } else {
        Chart::new()
            .kind(ChartKind::HorizontalBars)
            .swatch(Swatch::Black)
            .points(
                per_item
                    .into_iter()
                    .map(|(label, n)| {
                        ChartPoint::new(label, n as f64).display(
                            t!("stats.byItem.unit", count = n).unwrap_or_else(|_| n.to_string()),
                        )
                    })
                    .collect(),
            )
            .into()
    };

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
        Card::new()
            .title("i18n:stats.byItem.title")
            .subtitle("i18n:stats.byItem.subtitle")
            .child(chart)
            .into(),
        Card::new()
            .title("i18n:stats.feed.title")
            .subtitle("i18n:stats.feed.subtitle")
            .child(Stack::new().gap(0.0).children(rows))
            .into(),
    ])))
    .with_id(crate::ids::HOST_STATS)
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
