//! Les statistiques des listes du livret : ce que les voyageurs ont coché.

use chrono::{DateTime, Duration, Utc};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Chart, Grid, Page, Stack};
use portaki_sdk::sdui::surface::Surface;
use portaki_sdk::sdui::{ChartKind, ChartPoint};

use super::stats::*;

#[portaki_sdk::surface(
    host,
    id = "checklist",
    placement = HostPlacement::PropertyStatsCard,
    placement = HostPlacement::PropertyStatsDetail,
    label_key = "catalog.host.checklist",
    icon = IconName::CheckCircle
)]
pub fn render_stats_checklist(ctx: HostContext) -> Surface {
    let now = time::now().unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    let since = now - Duration::days(period_days(&ctx));
    let fr = is_fr(&ctx.locale);
    let tz = portaki_sdk::host::time::PropertyTz::parse(&ctx.property.timezone);
    let items = crate::domain::guest_items().unwrap_or_default();
    let stays = match period_stays(&ctx) {
        Some(period) => departures(
            &period,
            ticks_by_stay(since - Duration::days(30)).unwrap_or_default(),
            items.len(),
            since,
            now,
        ),
        None => guest_stays(since).unwrap_or_default(),
    };

    let body: Component = if stays.is_empty() {
        empty("i18n:stats.checklist.empty", IconName::CheckCircle)
    } else {
        let rates = item_rates(&items, |item| {
            stays
                .iter()
                .filter(|stay| stay.ticked.contains(&item.uuid()))
                .count()
        });
        let tiles = checklist_tiles(&stays, items.len(), &rates, fr);

        let recent: Vec<&GuestStay> = stays.iter().take(8).collect();
        let per_stay =
            recent
                .iter()
                .rev()
                .enumerate()
                .map(|(index, stay)| {
                    ChartPoint::new((index + 1).to_string(), stay.ticked.len() as f64)
                        .display(format!("{}/{}", stay.ticked.len(), stay.total))
                })
                .collect();

        let feed = stays
            .iter()
            .take(10)
            .map(|stay| stay_row(stay, fr, tz))
            .collect();

        Stack::new()
            .gap(16.0)
            .children(vec![
                Grid::new()
                    .minColumnWidth(170.0)
                    .gap(12.0)
                    .children(tiles)
                    .into(),
                panels(
                    (
                        "stats.checklist.byItem",
                        rate_chart(&rates, stays.len(), Swatch::Green),
                    ),
                    (
                        "stats.checklist.perStay",
                        Chart::new()
                            .kind(ChartKind::Bars)
                            .swatch(Swatch::Green)
                            .points(per_stay)
                            .into(),
                    ),
                ),
                feed_card("stats.checklist.feed", feed),
            ])
            .into()
    };
    Surface::new(Page::new().child(body)).with_id(CHECKLIST)
}
