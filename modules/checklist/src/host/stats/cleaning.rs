//! Les statistiques du ménage : les tâches d'équipe et leur échéance.

#[allow(unused_imports, reason = "the parent's imports, shared by its parts")]
use super::*;

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
            .map(|task| task_row(task, now, &ctx.locale))
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

/// « fini le 9 août · 13:20 », « … · 40 min de retard », « à terminer avant le 21 août · 14:00 ».
pub(super) fn cleaning_date(
    task: &CleaningTask<'_>,
    timing: Timing,
    at: DateTime<Utc>,
    fr: bool,
) -> Result<String> {
    let (date, time) = (short_date(at, fr), format_time(at));
    match (task.finished(), task.deadline(), task.done.is_empty()) {
        (true, Some(due), _) if timing == Timing::Late => t!(
            "stats.cleaning.row.lateBy",
            date = date,
            time = time,
            delay = format_duration(at - due)
        ),
        (true, _, _) => t!("stats.cleaning.row.doneAt", date = date, time = time),
        (false, Some(due), _) => t!(
            "stats.cleaning.row.dueAt",
            date = short_date(due, fr),
            time = format_time(due)
        ),
        (false, None, true) => Ok(String::new()),
        (false, None, false) => t!("stats.cleaning.row.lastAt", date = date, time = time),
    }
}

/// Finished tasks per week (30 and 90 days) or per month (12 months).
pub(super) fn per_week(
    tasks: &[&CleaningTask<'_>],
    now: DateTime<Utc>,
    days: i64,
    fr: bool,
) -> Component {
    let slices: i64 = if days == 365 { 12 } else { (days + 6) / 7 };
    let since = now - Duration::days(days);
    let mut counts = vec![0usize; slices as usize];
    for task in tasks {
        let Some(at) = task.last_at() else { continue };
        let index = ((at - since).num_seconds() * slices / Duration::days(days).num_seconds())
            .clamp(0, slices - 1);
        counts[index as usize] += 1;
    }
    let prefix = match (days, fr) {
        (365, _) => "M",
        (_, true) => "S",
        _ => "W",
    };
    Chart::new()
        .kind(ChartKind::Bars)
        .swatch(Swatch::Orange)
        .points(
            counts
                .iter()
                .enumerate()
                .map(|(i, n)| ChartPoint::new(format!("{prefix}{}", i + 1), *n as f64))
                .collect(),
        )
        .into()
}

/// One cleaning in the feed: when, who, how much, and against its deadline.
pub(super) fn task_row(task: &CleaningTask<'_>, now: DateTime<Utc>, locale: &str) -> Component {
    let fr = is_fr(locale);
    let at = task.last_at().unwrap_or(now);
    let timing = task.timing(now);
    let (label, tone) = match timing {
        Timing::OnTime => ("i18n:stats.cleaning.status.done", Tone::Success),
        Timing::Late => ("i18n:stats.cleaning.status.late", Tone::Danger),
        Timing::Open => ("i18n:stats.cleaning.status.open", Tone::Warning),
    };
    let name = task.list.name.get(locale);
    let mut meta = t!(
        "stats.cleaning.row.meta",
        done = task.done.len(),
        total = task.items.len()
    )
    .unwrap_or_default();
    if let (Some(assignee), _) = task.list.assignee() {
        meta = format!("{assignee} · {meta}");
    }
    if task.photos() > 0 {
        meta.push_str(&t!("stats.cleaning.row.photo").unwrap_or_default());
    }
    let title = match task.plan {
        Some((departure, _)) => {
            t!("stats.cleaning.row.after", date = short_date(departure, fr)).unwrap_or_default()
        }
        None => format!("{name} · {}", short_date(at, fr)),
    };
    let date = cleaning_date(task, timing, at, fr).unwrap_or_default();
    FeedItem::new()
        .title(title)
        .meta(meta)
        .date(date)
        .dotTone(tone)
        .status(FeedStatus::new(label, tone))
        .into()
}

/// « À temps » when deadlines are known, else « Terminés ».
pub(super) fn done_tile(
    tasks: &[CleaningTask<'_>],
    finished: usize,
    now: DateTime<Utc>,
    judged: bool,
) -> Component {
    // « À temps » sur les ménages déjà tranchés : finis, ou dont l'échéance est passée.
    let decided: Vec<Timing> = tasks
        .iter()
        .map(|task| task.timing(now))
        .filter(|timing| *timing != Timing::Open)
        .collect();
    let on_time = decided
        .iter()
        .filter(|timing| **timing == Timing::OnTime)
        .count();
    if judged {
        Stat::new()
            .label("i18n:stats.cleaning.onTime")
            .icon(IconName::CheckCircle)
            .value(percent(on_time, decided.len()))
            .delta("i18n:stats.cleaning.onTime.note")
            .into()
    } else {
        Stat::new()
            .label("i18n:stats.cleaning.done")
            .icon(IconName::CheckCircle)
            .value(percent(finished, tasks.len()))
            .delta(t!("stats.cleaning.done.note", count = tasks.len()).unwrap_or_default())
            .into()
    }
}
