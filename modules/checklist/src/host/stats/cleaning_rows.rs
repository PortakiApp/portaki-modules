//! Les lignes des statistiques du ménage : dates, semaines, fil, tuile « À temps ».

#[allow(unused_imports, reason = "the parent's imports, shared by its parts")]
use super::*;

/// « fini le 9 août · 13:20 », « … · 40 min de retard », « à terminer avant le 21 août · 14:00 ».
pub(crate) fn cleaning_date(
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
pub(crate) fn per_week(
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
pub(crate) fn task_row(task: &CleaningTask<'_>, now: DateTime<Utc>, locale: &str) -> Component {
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
pub(crate) fn done_tile(
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
