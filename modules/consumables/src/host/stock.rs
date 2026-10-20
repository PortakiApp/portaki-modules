//! The `property-stats-detail` surface (`stock`): tiles, charts and the catalog's stock rows.

use chrono::{DateTime, Duration, Utc};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, EmptyState, Grid, Page, Stack};
use portaki_sdk::sdui::surface::Surface;

use super::charts::{per_stay_chart, reports_chart, restock_chart, stock_row};
use super::stats::{departures, tiles};
use crate::entities::ConsumableReport;
use crate::{storage, Products};

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
