//! Property statistics « Synchro » — the tile (`statsSummary`) and the `property-stats-detail`
//! surface (design `tabStats` → `sync`).
//!
//! Reads the dashboard period from `input.periodDays` (30, 90 or 365). Tiles and charts come
//! from the KV sync snapshot: stays of the last feeds (dates, channel, email present) and the
//! per-day run history written by `applyFeeds`.

use chrono::{DateTime, Datelike, Duration, Utc};
use portaki_sdk::contracts::booking_channel::BookingChannel;
use portaki_sdk::contracts::stats::{self, AttentionLevel, StatsSummary, StatsSummaryArgs};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Chart, Grid, InfoBanner, Page, Stack, Stat};
use portaki_sdk::sdui::surface::Surface;
use portaki_sdk::sdui::{ChartKind, ChartPoint};

use crate::config::ModuleConfig;
use crate::i18n;
use crate::sync_state::{load_sync_state, SeenStay, SyncState};

/// Days shown by « Synchronisations par jour ».
const CHART_DAYS: i64 = 14;

/// Tile: time since the last sync, stays imported over the period, conflicts as attention.
#[portaki_sdk::query(name = "statsSummary")]
pub fn stats_summary(ctx: Context, args: StatsSummaryArgs) -> Result<StatsSummary> {
    let fr = ctx.locale.to_ascii_lowercase().starts_with("fr");
    let now = time::now()?;
    let state = load_sync_state()?;
    let value = state
        .last_run_at
        .as_deref()
        .and_then(parse)
        .map_or_else(|| "—".to_string(), |at| since(at, now, fr));
    let imported = imported_since(&state, now - Duration::days(i64::from(args.period)));
    let tile = stats::summary(
        value,
        i18n::text("stats.tile", &[("count", &imported.to_string())]),
    );
    let conflicts = count_conflicts(&upcoming(&state, now));
    if conflicts == 0 {
        return Ok(tile);
    }
    Ok(tile.attention(
        AttentionLevel::Action,
        i18n::text("stats.tile.conflicts", &[("count", &conflicts.to_string())]),
    ))
}

/// Stays not over yet, with their parsed dates.
fn upcoming(
    state: &SyncState,
    now: DateTime<Utc>,
) -> Vec<(&SeenStay, DateTime<Utc>, DateTime<Utc>)> {
    state
        .uids
        .values()
        .filter_map(|s| Some((s, parse(&s.check_in_at)?, parse(&s.check_out_at)?)))
        .filter(|(_, _, out)| *out > now)
        .collect()
}

#[portaki_sdk::surface(
    host,
    id = "calendar-sync",
    placement = HostPlacement::PropertyStatsCard,
    placement = HostPlacement::PropertyStatsDetail,
    label_key = "catalog.host.calendar-sync",
    icon = IconName::Calendar
)]
pub fn render_host_stats(ctx: HostContext) -> Result<Surface> {
    let fr = ctx.locale.to_ascii_lowercase().starts_with("fr");
    let days = period_days(&ctx);
    let now = time::now().unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    let config = ModuleConfig::load(&ctx)?;
    let state = load_sync_state().unwrap_or_default();
    let upcoming = upcoming(&state, now);

    let mut last_sync = Stat::new().label("i18n:stats.lastSync").value(
        state
            .last_run_at
            .as_deref()
            .and_then(parse)
            .map_or_else(|| "—".to_string(), |at| relative(at, now, fr)),
    );
    let sources: Vec<String> = config
        .connected_calendars()
        .iter()
        .map(|feed| {
            feed.label
                .clone()
                .filter(|l| !l.trim().is_empty())
                .unwrap_or_else(|| channel_name(feed.channel, fr).to_string())
        })
        .collect();
    if !sources.is_empty() {
        last_sync = last_sync.delta(sources.join(" · "));
    }

    let conflicts = count_conflicts(&upcoming);
    let incomplete = upcoming
        .iter()
        .filter(|(s, _, _)| !s.has_guest_email)
        .count();
    let tiles = Grid::new().minColumnWidth(170.0).gap(12.0).children(vec![
        last_sync.into(),
        Stat::new()
            .label("i18n:stats.imported")
            .value(imported_since(&state, now - Duration::days(days)).to_string())
            .delta(window_note(days, fr))
            .into(),
        with_note(
            Stat::new()
                .label("i18n:stats.conflicts")
                .value(conflicts.to_string()),
            conflicts > 0,
            "i18n:stats.conflicts.note",
        ),
        with_note(
            Stat::new()
                .label("i18n:stats.incomplete")
                .value(incomplete.to_string()),
            incomplete > 0,
            "i18n:stats.incomplete.note",
        ),
    ]);

    let panels = Grid::new().minColumnWidth(320.0).gap(16.0).children(vec![
        Card::new()
            .title("i18n:stats.runs.title")
            .subtitle("i18n:stats.runs.subtitle")
            .icon(IconName::Refresh)
            .children(vec![runs_chart(&state, now, fr)])
            .into(),
        Card::new()
            .title("i18n:stats.channels.title")
            .subtitle("i18n:stats.channels.subtitle")
            .icon(IconName::Calendar)
            .children(vec![channels_chart(&state, fr)])
            .into(),
    ]);

    let mut children: Vec<Component> = Vec::new();
    if summary_mentions_failures(state.summary.as_deref()) {
        children.push(
            InfoBanner::new()
                .message("i18n:stats.errorsHint")
                .tone(Tone::Warning)
                .into(),
        );
    }
    children.push(tiles.into());
    children.push(panels.into());

    Ok(
        Surface::new(Page::new().child(Stack::new().gap(16.0).children(children)))
            .with_id(crate::ids::HOST_STATS),
    )
}

fn with_note(stat: Stat, show: bool, note: &str) -> Component {
    if show { stat.delta(note) } else { stat }.into()
}

/// Dashboard period selector (`input.periodDays`): 30 j / 90 j / 12 mois, default 30.
fn period_days(ctx: &HostContext) -> i64 {
    match ctx.input_u64("periodDays") {
        Some(90) => 90,
        Some(365) => 365,
        _ => 30,
    }
}

fn parse(raw: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|at| at.with_timezone(&Utc))
}

fn day_key(at: DateTime<Utc>) -> String {
    at.format("%Y-%m-%d").to_string()
}

fn window_note(days: i64, fr: bool) -> String {
    match (days, fr) {
        (365, true) => "sur 12 mois".into(),
        (365, false) => "over 12 months".into(),
        (d, true) => format!("sur {d} jours"),
        (d, false) => format!("over {d} days"),
    }
}

/// « à l'instant », « il y a 12 min », « il y a 3 h », « il y a 2 j ».
fn relative(at: DateTime<Utc>, now: DateTime<Utc>, fr: bool) -> String {
    let minutes = (now - at).num_minutes().max(0);
    let (n, unit_fr, unit_en) = match minutes {
        0 => {
            return if fr {
                "à l'instant".into()
            } else {
                "just now".into()
            }
        }
        m if m < 60 => (m, "min", "min"),
        m if m < 48 * 60 => (m / 60, "h", "h"),
        m => (m / (24 * 60), "j", "d"),
    };
    if fr {
        format!("il y a {n} {unit_fr}")
    } else {
        format!("{n} {unit_en} ago")
    }
}

/// Tile value, at most 12 characters: « 12 min », « 3 h », « 2 j ».
fn since(at: DateTime<Utc>, now: DateTime<Utc>, fr: bool) -> String {
    match (now - at).num_minutes().max(0) {
        m if m < 60 => format!("{m} min"),
        m if m < 48 * 60 => format!("{} h", m / 60),
        m => format!("{} {}", m / (24 * 60), if fr { "j" } else { "d" }),
    }
}

/// Stays first seen on or after `since` (run history, whole days).
fn imported_since(state: &SyncState, since: DateTime<Utc>) -> u32 {
    let from = day_key(since);
    state
        .history
        .range(from..)
        .map(|(_, runs)| runs.new_stays)
        .sum()
}

/// Pairs of still-running or upcoming stays whose nights overlap.
fn count_conflicts(stays: &[(&SeenStay, DateTime<Utc>, DateTime<Utc>)]) -> usize {
    let mut pairs = 0;
    for (i, (_, in_a, out_a)) in stays.iter().enumerate() {
        for (_, in_b, out_b) in &stays[i + 1..] {
            if in_a < out_b && in_b < out_a {
                pairs += 1;
            }
        }
    }
    pairs
}

/// Last [`CHART_DAYS`] days, runs per day; the most recent day with a failure is highlighted.
fn runs_chart(state: &SyncState, now: DateTime<Utc>, fr: bool) -> Component {
    let days: Vec<DateTime<Utc>> = (0..CHART_DAYS)
        .rev()
        .map(|back| now - Duration::days(back))
        .collect();
    let mut highlight = days.len() - 1;
    let points = days
        .iter()
        .enumerate()
        .map(|(i, day)| {
            let runs = state
                .history
                .get(&day_key(*day))
                .copied()
                .unwrap_or_default();
            if runs.failed > 0 {
                highlight = i;
            }
            let display = if fr {
                let plural = if runs.ok > 1 { "s" } else { "" };
                format!("{} réussie{plural} · {} en échec", runs.ok, runs.failed)
            } else {
                format!("{} ok · {} failed", runs.ok, runs.failed)
            };
            ChartPoint::new(day.day().to_string(), f64::from(runs.ok + runs.failed))
                .display(display)
        })
        .collect();
    Chart::new()
        .kind(ChartKind::Bars)
        .swatch(Swatch::Blue)
        .highlight(u32::try_from(highlight).unwrap_or(0))
        .points(points)
        .into()
}

/// Stays of the last snapshot, per booking platform, most first.
fn channels_chart(state: &SyncState, fr: bool) -> Component {
    let mut counts: Vec<(BookingChannel, usize)> = BookingChannel::ALL
        .iter()
        .map(|c| (*c, state.uids.values().filter(|s| s.channel == *c).count()))
        .filter(|(_, n)| *n > 0)
        .collect();
    counts.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    let points = counts
        .into_iter()
        .map(|(channel, n)| {
            let unit = match (n, fr) {
                (1, true) => "séjour",
                (_, true) => "séjours",
                (1, false) => "stay",
                (_, false) => "stays",
            };
            ChartPoint::new(channel_name(channel, fr), n as f64).display(format!("{n} {unit}"))
        })
        .collect();
    Chart::new()
        .kind(ChartKind::HorizontalBars)
        .swatch(Swatch::Blue)
        .points(points)
        .into()
}

fn channel_name(channel: BookingChannel, fr: bool) -> &'static str {
    match (channel, fr) {
        (BookingChannel::Airbnb, _) => "Airbnb",
        (BookingChannel::Booking, _) => "Booking.com",
        (BookingChannel::AbritelVrbo, _) => "Abritel / Vrbo",
        (BookingChannel::Direct, true) => "Réservation directe",
        (BookingChannel::Direct, false) => "Direct booking",
        (BookingChannel::OtherPlatform, true) => "Autre plateforme",
        (BookingChannel::OtherPlatform, false) => "Other platform",
        (BookingChannel::Unknown, true) => "Plateforme inconnue",
        (BookingChannel::Unknown, false) => "Unknown platform",
    }
}

fn summary_mentions_failures(summary: Option<&str>) -> bool {
    let Some(raw) = summary.map(str::trim).filter(|s| !s.is_empty()) else {
        return false;
    };
    // Summary format: "N stay(s) · X feed(s) ok · Y feed(s) failed"
    for part in raw.split('·') {
        let part = part.trim();
        if !part.contains("failed") {
            continue;
        }
        let digits: String = part.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(n) = digits.parse::<u32>() {
            return n > 0;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(raw: &str) -> DateTime<Utc> {
        parse(raw).unwrap()
    }

    fn stay(check_in: &str, check_out: &str) -> SeenStay {
        SeenStay {
            check_in_at: check_in.into(),
            check_out_at: check_out.into(),
            ..SeenStay::default()
        }
    }

    #[test]
    fn overlapping_stays_count_as_one_conflict_per_pair() {
        let a = stay("2026-10-01T15:00:00Z", "2026-10-05T10:00:00Z");
        let b = stay("2026-10-04T15:00:00Z", "2026-10-06T10:00:00Z");
        let c = stay("2026-10-06T15:00:00Z", "2026-10-08T10:00:00Z");
        let rows: Vec<_> = [&a, &b, &c]
            .into_iter()
            .map(|s| (s, at(&s.check_in_at), at(&s.check_out_at)))
            .collect();
        // b leaves at 10:00 the day c arrives at 15:00 — back to back, not a conflict.
        assert_eq!(count_conflicts(&rows), 1);
    }

    #[test]
    fn imported_sums_new_stays_inside_the_window() {
        let mut state = SyncState::default();
        state.record_run("2026-07-01", false, 5);
        state.record_run("2026-09-10", false, 2);
        state.record_run("2026-09-20", true, 1);
        let now = at("2026-09-23T00:00:00Z");
        assert_eq!(imported_since(&state, now - Duration::days(30)), 3);
        assert_eq!(imported_since(&state, now - Duration::days(90)), 8);
    }

    #[test]
    fn relative_times() {
        let now = at("2026-09-23T12:00:00Z");
        assert_eq!(
            relative(now - Duration::minutes(12), now, true),
            "il y a 12 min"
        );
        assert_eq!(relative(now - Duration::hours(3), now, true), "il y a 3 h");
        assert_eq!(relative(now - Duration::days(3), now, false), "3 d ago");
    }
}
