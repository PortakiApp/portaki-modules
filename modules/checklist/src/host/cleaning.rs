//! Les statistiques du ménage : les tâches d'équipe et leur échéance.

use chrono::{DateTime, Duration, Utc};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Grid, Page, Stack, Stat};
use portaki_sdk::sdui::surface::Surface;

use super::stats::*;
use crate::Step;

#[portaki_sdk::surface(
    host,
    id = "cleaning",
    placement = HostPlacement::PropertyStatsCard,
    placement = HostPlacement::PropertyStatsDetail,
    label_key = "catalog.host.cleaning",
    icon = IconName::Sparkles
)]
pub fn render_stats_cleaning(ctx: HostContext) -> Surface {
    let now = time::now().unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    let days = period_days(&ctx);
    let since = now - Duration::days(days);
    let fr = is_fr(&ctx.locale);
    let tz = portaki_sdk::host::time::PropertyTz::parse(&ctx.property.timezone);
    let data = cleaning_data().unwrap_or(CleaningData {
        lists: Vec::new(),
        items: Vec::new(),
        states: Vec::new(),
    });
    let tasks = match period_stays(&ctx) {
        Some(stays) => planned_cleanings(&data, &stays, &ctx.property.timezone, since, now),
        None => cleaning_tasks_of(&data, since),
    };
    let judged = tasks.iter().any(|task| task.deadline().is_some());

    let body: Component = if tasks.is_empty() {
        empty("i18n:stats.cleaning.empty", IconName::Sparkles)
    } else {
        let finished: Vec<&CleaningTask<'_>> = tasks.iter().filter(|t| t.finished()).collect();
        let durations: Vec<Duration> = finished
            .iter()
            .filter_map(|task| Some(task.last_at()? - task.first_at()?))
            .collect();
        // ponytail: rates over every cleaning task; per list if a property runs several.
        let items: Vec<Step> = data.items.iter().map(|(_, step)| step.clone()).collect();
        let rates = item_rates(&items, |item| {
            tasks
                .iter()
                .filter(|task| task.done.iter().any(|state| state.item_id == item.uuid()))
                .count()
        });
        let with_photo = finished.iter().filter(|task| task.photos() > 0).count();

        let mut tiles = vec![done_tile(&tasks, finished.len(), now, judged)];
        tiles.extend([Stat::new()
            .label("i18n:stats.cleaning.duration")
            .icon(IconName::ClockCircle)
            .value(average(&durations).map_or_else(|| "—".to_string(), format_duration))
            .delta("i18n:stats.cleaning.duration.note")
            .into()]);
        tiles.extend(most_forgotten(&rates, tasks.len(), fr, "stats.cleaning"));
        tiles.push(
            Stat::new()
                .label("i18n:stats.cleaning.photos")
                .icon(IconName::Image)
                .value(format!("{with_photo} / {}", finished.len()))
                .delta("i18n:stats.cleaning.photos.note")
                .into(),
        );

        let feed = tasks
            .iter()
            .take(10)
            .map(|task| task_row(task, now, &ctx.locale, tz))
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
                    ("stats.cleaning.perWeek", per_week(&finished, now, days, fr)),
                    (
                        "stats.cleaning.byItem",
                        rate_chart(&rates, tasks.len(), Swatch::Black),
                    ),
                ),
                feed_card("stats.cleaning.feed", feed),
            ])
            .into()
    };
    Surface::new(Page::new().child(body)).with_id(CLEANING)
}
