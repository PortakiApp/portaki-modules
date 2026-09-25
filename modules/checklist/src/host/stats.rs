//! Statistics — tiles (`statsSummary`) and details for « Checklist » (guest lists) and
//! « Ménage » (host cleaning lists).
//!
//! Only what the module stores is shown: guest ticks per stay, host ticks per task. The platform
//! passes the period's stays to the detail page (`input.stays` — dates and status, no guest name):
//! with them, a stay that never opened its list shows as « Non remplie » and a cleaning is judged
//! against its deadline. Without them (developer preview, older platform), both stay unknown.

use std::collections::BTreeMap;

use chrono::{DateTime, Duration, Utc};
use portaki_sdk::contracts::stats::{self, AttentionLevel, StatsSummary, StatsSummaryArgs};
use portaki_sdk::contracts::timeline::TimelineStay;
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Chart, EmptyState, FeedItem, Grid, Page, Stack, Stat};
use portaki_sdk::sdui::surface::Surface;
use portaki_sdk::sdui::{ChartKind, ChartPoint, FeedStatus};
use uuid::Uuid;

use crate::entities::{Checklist, ChecklistItem, TaskItemState};
use crate::labels;
use crate::lists;
use crate::tasks::{parse_task_id, plan_tasks, task_id};
use crate::{i18n, storage};

#[portaki_sdk::query(
    name = "statsSummary",
    example(
        label = "Checklists sur 30 jours",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","period":30,"key":"checklist"}"#
    ),
    example(
        label = "Ménage sur 90 jours",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","period":90,"key":"cleaning"}"#
    )
)]
pub fn stats_summary(_ctx: Context, args: StatsSummaryArgs) -> Result<StatsSummary> {
    let since = time::now()? - Duration::days(i64::from(args.period));
    match args.key.as_str() {
        "checklist" => {
            let stays = guest_stays(since)?;
            let value = percent(complete_count(&stays), stays.len());
            Ok(stats::summary(
                value,
                i18n::text("stats.checklist.tile", &[]),
            ))
        }
        "cleaning" => {
            let data = cleaning_data()?;
            let tasks = cleaning_tasks_of(&data, since);
            let open = tasks.iter().filter(|task| !task.finished()).count();
            let finished = tasks.len() - open;
            let tile = stats::summary(
                percent(finished, tasks.len()),
                i18n::text("stats.cleaning.tile", &[]),
            );
            if open == 0 {
                return Ok(tile);
            }
            let count = open.to_string();
            Ok(tile.attention(
                AttentionLevel::Action,
                i18n::text("stats.cleaning.attention", &[("count", &count)]),
            ))
        }
        other => Err(PortakiError::Host(format!("unknown_stats_key:{other}"))),
    }
}

fn percent(part: usize, whole: usize) -> String {
    match (part * 100).checked_div(whole) {
        Some(value) => format!("{value} %"),
        None => "—".to_string(),
    }
}

/// Dashboard period selector: 30 j / 90 j / 12 mois; anything else falls back to 30.
fn period_days(ctx: &HostContext) -> i64 {
    match ctx.input_u64("periodDays") {
        Some(90) => 90,
        Some(365) => 365,
        _ => 30,
    }
}

/// The period's stays the platform passes to the detail page, oldest first; `None` when it sends
/// none (developer preview, older platform).
fn period_stays(ctx: &HostContext) -> Option<Vec<TimelineStay>> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Raw {
        id: Uuid,
        check_in: DateTime<Utc>,
        check_out: DateTime<Utc>,
        #[serde(default)]
        status: String,
    }
    let raw: Vec<Raw> = serde_json::from_value(ctx.input.get("stays")?.clone()).ok()?;
    let mut stays: Vec<TimelineStay> = raw
        .into_iter()
        .map(|stay| TimelineStay {
            id: stay.id,
            check_in: stay.check_in,
            check_out: stay.check_out,
            guest_name: String::new(),
            status: stay.status,
        })
        .collect();
    stays.sort_by_key(|stay| stay.check_in);
    Some(stays)
}

// --- Checklist (guest) ----------------------------------------------------------------------

/// Guest ticks of one stay over the guest items that still exist.
struct GuestStay {
    ticked: Vec<Uuid>,
    total: usize,
    last_at: DateTime<Utc>,
    /// `(check-in, check-out)` when the platform passed the stay.
    window: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

impl GuestStay {
    fn complete(&self) -> bool {
        self.ticked.len() >= self.total
    }
}

fn complete_count(stays: &[GuestStay]) -> usize {
    stays.iter().filter(|stay| stay.complete()).count()
}

/// Guest ticks per stay since `since`.
fn ticks_by_stay(since: DateTime<Utc>) -> Result<BTreeMap<Uuid, GuestStay>> {
    let items = crate::queries::guest_items()?;
    let mut by_stay: BTreeMap<Uuid, GuestStay> = BTreeMap::new();
    for row in storage::list_completions(None)? {
        if row.completed_at < since || !items.iter().any(|item| item.id == row.item_id) {
            continue;
        }
        let stay = by_stay.entry(row.stay_id).or_insert(GuestStay {
            ticked: Vec::new(),
            total: items.len(),
            last_at: row.completed_at,
            window: None,
        });
        stay.ticked.push(row.item_id);
        stay.last_at = stay.last_at.max(row.completed_at);
    }
    Ok(by_stay)
}

/// Stays that ticked something since `since`, newest first.
fn guest_stays(since: DateTime<Utc>) -> Result<Vec<GuestStay>> {
    let mut stays: Vec<GuestStay> = ticks_by_stay(since)?.into_values().collect();
    stays.sort_by_key(|stay| std::cmp::Reverse(stay.last_at));
    Ok(stays)
}

/// Every departure of `(since, now]`, newest first, with what its guest ticked — none for a list
/// never opened. Ticks are read from a week before the period: a guest ticks during the stay.
fn departures(
    stays: &[TimelineStay],
    mut ticks: BTreeMap<Uuid, GuestStay>,
    total: usize,
    since: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Vec<GuestStay> {
    let mut out: Vec<GuestStay> = stays
        .iter()
        .filter(|stay| stay.check_out > since && stay.check_out <= now)
        .map(|stay| {
            let mut guest = ticks.remove(&stay.id).unwrap_or(GuestStay {
                ticked: Vec::new(),
                total,
                last_at: stay.check_out,
                window: None,
            });
            guest.window = Some((stay.check_in, stay.check_out));
            guest
        })
        .collect();
    out.sort_by_key(|stay| std::cmp::Reverse(stay.window.map(|(_, out)| out)));
    out
}

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
    let fr = labels::lang_code(&ctx.locale) == "fr";
    let items = crate::queries::guest_items().unwrap_or_default();
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
                .filter(|stay| stay.ticked.contains(&item.id))
                .count()
        });
        let ticked: usize = stays.iter().map(|stay| stay.ticked.len()).sum();
        let average = ticked as f64 / stays.len() as f64;
        let mut tiles = vec![
            Stat::new()
                .label("i18n:stats.checklist.completed")
                .icon(IconName::CheckCircle)
                .value(percent(complete_count(&stays), stays.len()))
                .delta(
                    t!("stats.checklist.completed.note", count = stays.len()).unwrap_or_default(),
                )
                .into(),
            Stat::new()
                .label("i18n:stats.checklist.ticked")
                .icon(IconName::Clipboard)
                .value(format!("{} / {}", decimal(average, fr), items.len()))
                .delta("i18n:stats.checklist.ticked.note")
                .into(),
        ];
        tiles.extend(most_forgotten(&rates, stays.len(), fr, "stats.checklist"));

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
            .map(|stay| {
                let (label, tone) = if stay.complete() {
                    ("i18n:stats.checklist.status.complete", Tone::Success)
                } else if stay.ticked.is_empty() {
                    ("i18n:stats.checklist.status.unfilled", Tone::Neutral)
                } else {
                    ("i18n:stats.checklist.status.incomplete", Tone::Warning)
                };
                let title = match stay.window {
                    Some((check_in, check_out)) => t!(
                        "stats.checklist.row.stay",
                        from = short_date(check_in, fr),
                        to = short_date(check_out, fr)
                    ),
                    None => t!(
                        "stats.checklist.row.title",
                        date = short_date(stay.last_at, fr)
                    ),
                }
                .unwrap_or_default();
                let date = match stay.window {
                    Some((_, check_out)) => t!(
                        "stats.checklist.row.departure",
                        date = short_date(check_out, fr)
                    )
                    .unwrap_or_default(),
                    None => format_time(stay.last_at),
                };
                FeedItem::new()
                    .title(title)
                    .meta(
                        t!(
                            "stats.checklist.row.meta",
                            done = stay.ticked.len(),
                            total = stay.total
                        )
                        .unwrap_or_default(),
                    )
                    .date(date)
                    .dotTone(tone)
                    .status(FeedStatus::new(label, tone))
                    .into()
            })
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

// --- Ménage (host) --------------------------------------------------------------------------

/// Ticks of one task of a host cleaning list.
struct CleaningTask<'a> {
    list: &'a Checklist,
    items: Vec<&'a ChecklistItem>,
    done: Vec<&'a TaskItemState>,
    /// `(departure, deadline)` when the platform passed the stays.
    plan: Option<(DateTime<Utc>, Option<DateTime<Utc>>)>,
}

/// Where a cleaning stands against its deadline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Timing {
    Open,
    OnTime,
    /// Finished after its deadline, or not finished once it has passed.
    Late,
}

impl CleaningTask<'_> {
    fn finished(&self) -> bool {
        self.done.len() >= self.items.len()
    }

    fn deadline(&self) -> Option<DateTime<Utc>> {
        self.plan.and_then(|(_, due)| due)
    }

    fn timing(&self, now: DateTime<Utc>) -> Timing {
        match (self.finished(), self.deadline(), self.last_at()) {
            (true, Some(due), Some(at)) if at > due => Timing::Late,
            (true, _, _) => Timing::OnTime,
            (false, Some(due), _) if now > due => Timing::Late,
            (false, _, _) => Timing::Open,
        }
    }

    fn first_at(&self) -> Option<DateTime<Utc>> {
        self.done.iter().filter_map(|state| state.done_at).min()
    }

    fn last_at(&self) -> Option<DateTime<Utc>> {
        self.done.iter().filter_map(|state| state.done_at).max()
    }

    fn photos(&self) -> usize {
        self.done
            .iter()
            .filter(|state| state.photo.is_some())
            .count()
    }
}

/// Everything `CleaningTask` borrows from, loaded once.
struct CleaningData {
    lists: Vec<Checklist>,
    items: Vec<ChecklistItem>,
    states: Vec<TaskItemState>,
}

fn cleaning_data() -> Result<CleaningData> {
    let lists = storage::list_checklists()?
        .into_iter()
        .filter(|list| {
            list.audience == lists::HOST
                && [lists::AFTER_EACH_DEPARTURE, lists::ONLY_IF_NEXT_ARRIVAL]
                    .contains(&list.trigger.as_str())
        })
        .collect();
    Ok(CleaningData {
        lists,
        items: storage::list_items()?,
        states: storage::task_states(None)?,
    })
}

/// Tasks ticked since `since`, most recent activity first.
fn cleaning_tasks_of(data: &CleaningData, since: DateTime<Utc>) -> Vec<CleaningTask<'_>> {
    let mut by_task: BTreeMap<&str, CleaningTask<'_>> = BTreeMap::new();
    for state in data.states.iter().filter(|state| state.done) {
        let Some(list) = parse_task_id(&state.task_id)
            .and_then(|(list_id, _)| data.lists.iter().find(|list| list.id == list_id))
        else {
            continue;
        };
        let task = by_task
            .entry(&state.task_id)
            .or_insert_with(|| CleaningTask {
                list,
                items: data
                    .items
                    .iter()
                    .filter(|item| item.checklist_id == list.id)
                    .collect(),
                done: Vec::new(),
                plan: None,
            });
        if task.items.iter().any(|item| item.id == state.item_id) {
            task.done.push(state);
        }
    }
    let mut tasks: Vec<CleaningTask<'_>> = by_task
        .into_values()
        .filter(|task| task.last_at().is_some_and(|at| at >= since))
        .collect();
    tasks.sort_by_key(|task| std::cmp::Reverse(task.last_at()));
    tasks
}

/// One task per departure of `(since, now]` the list's trigger applies to — ticked or not — with
/// its deadline, most recent departure first.
fn planned_cleanings<'a>(
    data: &'a CleaningData,
    stays: &[TimelineStay],
    timezone: &str,
    since: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Vec<CleaningTask<'a>> {
    let mut tasks = Vec::new();
    for list in &data.lists {
        let items: Vec<&ChecklistItem> = data
            .items
            .iter()
            .filter(|item| item.checklist_id == list.id)
            .collect();
        if items.is_empty() {
            continue;
        }
        for plan in plan_tasks(list, stays, timezone) {
            if plan.at <= since || plan.at > now {
                continue;
            }
            let id = task_id(list.id, plan.stay_id);
            let done = data
                .states
                .iter()
                .filter(|state| {
                    state.done
                        && state.task_id == id
                        && items.iter().any(|item| item.id == state.item_id)
                })
                .collect();
            tasks.push(CleaningTask {
                list,
                items: items.clone(),
                done,
                plan: Some((plan.at, plan.due_at)),
            });
        }
    }
    tasks.sort_by_key(|task| std::cmp::Reverse(task.plan.map(|(at, _)| at)));
    tasks
}

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
    let fr = labels::lang_code(&ctx.locale) == "fr";
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
        let items: Vec<ChecklistItem> = data
            .items
            .iter()
            .filter(|item| data.lists.iter().any(|list| list.id == item.checklist_id))
            .cloned()
            .collect();
        let rates = item_rates(&items, |item| {
            tasks
                .iter()
                .filter(|task| task.done.iter().any(|state| state.item_id == item.id))
                .count()
        });
        let with_photo = finished.iter().filter(|task| task.photos() > 0).count();

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
        let mut tiles = vec![if judged {
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
                .value(percent(finished.len(), tasks.len()))
                .delta(t!("stats.cleaning.done.note", count = tasks.len()).unwrap_or_default())
                .into()
        }];
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
            .map(|task| {
                let at = task.last_at().unwrap_or(now);
                let timing = task.timing(now);
                let (label, tone) = match timing {
                    Timing::OnTime => ("i18n:stats.cleaning.status.done", Tone::Success),
                    Timing::Late => ("i18n:stats.cleaning.status.late", Tone::Danger),
                    Timing::Open => ("i18n:stats.cleaning.status.open", Tone::Warning),
                };
                let name = if fr {
                    &task.list.name_fr
                } else {
                    &task.list.name_en
                };
                let mut meta = t!(
                    "stats.cleaning.row.meta",
                    done = task.done.len(),
                    total = task.items.len()
                )
                .unwrap_or_default();
                if let Some(assignee) = &task.list.assignee_name {
                    meta = format!("{assignee} · {meta}");
                }
                if task.photos() > 0 {
                    meta.push_str(&t!("stats.cleaning.row.photo").unwrap_or_default());
                }
                let title = match task.plan {
                    Some((departure, _)) => {
                        t!("stats.cleaning.row.after", date = short_date(departure, fr))
                            .unwrap_or_default()
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
            })
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
fn cleaning_date(
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
fn per_week(tasks: &[&CleaningTask<'_>], now: DateTime<Utc>, days: i64, fr: bool) -> Component {
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

// --- Shared ---------------------------------------------------------------------------------

/// `(item, how many stays or tasks ticked it)`, least ticked last.
fn item_rates(
    items: &[ChecklistItem],
    ticked: impl Fn(&ChecklistItem) -> usize,
) -> Vec<(&ChecklistItem, usize)> {
    let mut rates: Vec<(&ChecklistItem, usize)> =
        items.iter().map(|item| (item, ticked(item))).collect();
    rates.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    rates
}

fn rate_chart(rates: &[(&ChecklistItem, usize)], of: usize, swatch: Swatch) -> Component {
    Chart::new()
        .kind(ChartKind::HorizontalBars)
        .swatch(swatch)
        .max(100.0)
        .points(
            rates
                .iter()
                .map(|(item, n)| {
                    let rate = (n * 100).checked_div(of).unwrap_or(0);
                    ChartPoint::new(labels::i18n_label(item).fr, rate as f64)
                        .display(format!("{rate} %"))
                })
                .collect(),
        )
        .into()
}

/// « La plus oubliée » — only once something was missed.
fn most_forgotten(
    rates: &[(&ChecklistItem, usize)],
    of: usize,
    fr: bool,
    prefix: &str,
) -> Option<Component> {
    let (item, ticked) = rates.last().filter(|(_, n)| *n < of)?;
    let label = labels::i18n_label(item);
    Some(
        Stat::new()
            .label(format!("i18n:{prefix}.forgotten"))
            .icon(IconName::DangerTriangle)
            .value(if fr { label.fr } else { label.en })
            .delta(
                t!(
                    &format!("{prefix}.forgotten.note"),
                    missed = of - ticked,
                    count = of
                )
                .unwrap_or_default(),
            )
            .into(),
    )
}

fn panels(left: (&str, Component), right: (&str, Component)) -> Component {
    let card = |(key, body): (&str, Component)| -> Component {
        Card::new()
            .title(format!("i18n:{key}.title"))
            .subtitle(format!("i18n:{key}.subtitle"))
            .child(body)
            .into()
    };
    Grid::new()
        .minColumnWidth(320.0)
        .gap(16.0)
        .children(vec![card(left), card(right)])
        .into()
}

fn feed_card(key: &str, rows: Vec<Component>) -> Component {
    Card::new()
        .title(format!("i18n:{key}.title"))
        .subtitle(format!("i18n:{key}.subtitle"))
        .child(Stack::new().gap(0.0).children(rows))
        .into()
}

fn empty(description: &str, icon: IconName) -> Component {
    EmptyState::new()
        .title("i18n:stats.empty")
        .description(description)
        .icon(icon)
        .into()
}

fn average(durations: &[Duration]) -> Option<Duration> {
    let n = i32::try_from(durations.len()).ok().filter(|n| *n > 0)?;
    Some(durations.iter().copied().sum::<Duration>() / n)
}

/// « 40 min », « 2 h 10 ».
fn format_duration(duration: Duration) -> String {
    let minutes = duration.num_minutes().max(0);
    match (minutes / 60, minutes % 60) {
        (0, m) => format!("{m} min"),
        (h, 0) => format!("{h} h"),
        (h, m) => format!("{h} h {m:02}"),
    }
}

fn decimal(value: f64, fr: bool) -> String {
    let text = format!("{value:.1}");
    if fr {
        text.replace('.', ",")
    } else {
        text
    }
}

fn format_time(at: DateTime<Utc>) -> String {
    at.format("%H:%M").to_string()
}

fn short_date(at: DateTime<Utc>, fr: bool) -> String {
    use chrono::Datelike;
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

    #[test]
    fn formats() {
        assert_eq!(format_duration(Duration::minutes(130)), "2 h 10");
        assert_eq!(format_duration(Duration::minutes(40)), "40 min");
        assert_eq!(format_duration(Duration::minutes(120)), "2 h");
        assert_eq!(decimal(5.14, true), "5,1");
        assert_eq!(percent(7, 9), "77 %");
        assert_eq!(percent(0, 0), "—");
    }
}
