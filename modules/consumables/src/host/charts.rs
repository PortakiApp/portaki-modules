//! Les graphiques et les lignes du détail des statistiques.

use chrono::{DateTime, Datelike, Utc};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Chart, EmptyState, FeedItem};
use portaki_sdk::sdui::{ChartKind, ChartPoint, FeedStatus};
use uuid::Uuid;

use super::stats::{days_between_restocks, label, stock, Stock};
use crate::entities::ConsumableReport;
use crate::{status, Product};

pub(super) fn panel(key: &str, body: Component) -> Component {
    Card::new()
        .title(format!("i18n:stats.{key}.title"))
        .subtitle(format!("i18n:stats.{key}.subtitle"))
        .child(body)
        .into()
}

pub(super) fn bars(swatch: Swatch, points: Vec<ChartPoint>) -> Component {
    Chart::new()
        .kind(ChartKind::HorizontalBars)
        .swatch(swatch)
        .points(points)
        .into()
}

pub(super) fn nothing(description: &str) -> Component {
    EmptyState::new()
        .title("i18n:stats.empty")
        .description(description)
        .icon(IconName::Package)
        .into()
}

/// Share of the period's departures where a guest reported the item missing or low.
pub(super) fn per_stay_chart(
    items: &[Product],
    period: &[&ConsumableReport],
    stays: &[Uuid],
    locale: &str,
) -> Component {
    let mut rates: Vec<(String, usize)> = items
        .iter()
        .map(|item| {
            let mut short: Vec<Uuid> = period
                .iter()
                .filter(|r| r.item_id == item.uuid() && stays.contains(&r.stay_id))
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
pub(super) fn reports_chart(
    items: &[Product],
    period: &[&ConsumableReport],
    locale: &str,
) -> Component {
    let mut per_item: Vec<(String, usize)> = items
        .iter()
        .map(|item| {
            let n = period.iter().filter(|r| r.item_id == item.uuid()).count();
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
pub(super) fn restock_chart(
    items: &[Product],
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

pub(super) fn stock_row(
    item: &Product,
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
    let mine = || reports.iter().filter(|r| r.item_id == item.uuid());
    let open = mine().find(|r| status::is_pending(&r.status));
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

pub(super) fn short_date(at: DateTime<Utc>, fr: bool) -> String {
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
