//! Property statistics « Signalements » — the tile (`statsSummary`) and the
//! `property-stats-detail` surface (design `tabStats` → `issues`).
//!
//! The dashboard passes the selected period as `input.periodDays` (30, 90 or 365).
//! « Avec photo » counts the period's reports that carry a guest photo. The recent reports feed
//! sits under the panels — the module has no config tab.

use chrono::{DateTime, Datelike, Duration, Utc};
use portaki_sdk::contracts::stats::{self, AttentionLevel, StatsSummary, StatsSummaryArgs};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Chart, EmptyState, Grid, Page, Stack, Stat};
use portaki_sdk::sdui::surface::Surface;
use portaki_sdk::sdui::{ChartKind, ChartPoint};

use uuid::Uuid;

use crate::category;
use crate::entities::IssueReport;
use crate::{i18n, storage};

/// Tile: reports of the period, and the open ones to handle.
#[portaki_sdk::query(
    name = "statsSummary",
    example(
        label = "Signalements sur 30 jours",
        input = r#"{"propertyId":"8c0e6f2a-1d3b-4a7e-b5c9-0f4e2d1a6b38","period":30,"key":"issue-stats"}"#
    )
)]
pub fn stats_summary(_ctx: Context, args: StatsSummaryArgs) -> Result<StatsSummary> {
    let days = i64::from(args.period);
    let reports = storage::list_since(time::now()? - Duration::days(days))?;
    let open = reports.iter().filter(|r| r.resolved_at.is_none()).count();
    let tile = stats::summary(
        reports.len().to_string(),
        i18n::text(&format!("stats.tile.{}", period_key(days)), &[]),
    );
    if open == 0 {
        return Ok(tile);
    }
    let count = open.to_string();
    Ok(tile.attention(
        AttentionLevel::Action,
        i18n::text("stats.tile.open", &[("count", &count)]),
    ))
}

/// Bundle suffix of a period: 30, 90 or 365 (anything else reads as 30).
fn period_key(days: i64) -> i64 {
    match days {
        90 | 365 => days,
        _ => 30,
    }
}

#[portaki_sdk::surface(
    host,
    id = "issue-stats",
    placement = HostPlacement::PropertyStatsCard,
    placement = HostPlacement::PropertyStatsDetail,
    label_key = "catalog.host.issue-stats",
    icon = IconName::DangerTriangle
)]
pub fn render_host_stats(ctx: HostContext) -> Surface {
    let fr = ctx.locale.to_ascii_lowercase().starts_with("fr");
    let days = period_days(&ctx);
    let now = super::host_now();
    // The dashboard re-renders this surface in its detail modal with the row's `issueId`.
    if let Some(report) = ctx
        .input_str("issueId")
        .and_then(|id| Uuid::parse_str(id).ok())
        .and_then(|id| storage::find_by_id(id).ok().flatten())
    {
        return Surface::new(super::report_detail(&report, &ctx.locale))
            .with_id(crate::ids::HOST_STATS);
    }
    let reports = storage::list_since(now - Duration::days(days)).unwrap_or_default();

    let delays: Vec<Duration> = reports
        .iter()
        .filter_map(|r| r.resolved_at.map(|at| at - r.created_at))
        .collect();
    let open_reports: Vec<&IssueReport> =
        reports.iter().filter(|r| r.resolved_at.is_none()).collect();

    let mut resolved = Stat::new()
        .label("i18n:stats.resolved")
        .icon(IconName::CheckCircle)
        .value(delays.len().to_string());
    if let Some(avg) = average(&delays) {
        let prefix = if fr { "délai moyen" } else { "avg." };
        resolved = resolved.delta(format!("{prefix} {}", format_delay(avg, fr)));
    }
    let mut open = Stat::new()
        .label("i18n:stats.open")
        .icon(IconName::ClockCircle)
        .value(open_reports.len().to_string());
    if let Some(newest) = open_reports.iter().max_by_key(|r| r.created_at) {
        open = open.delta(newest.summary.clone());
    }

    let tiles = Grid::new().minColumnWidth(170.0).gap(12.0).children(vec![
        Stat::new()
            .label("i18n:stats.reports")
            .icon(IconName::DangerTriangle)
            .value(reports.len().to_string())
            .delta(window_note(days, fr))
            .into(),
        resolved.into(),
        open.into(),
        Stat::new()
            .label("i18n:stats.withPhoto")
            .value(
                reports
                    .iter()
                    .filter(|r| r.photo.is_some())
                    .count()
                    .to_string(),
            )
            .delta("i18n:stats.withPhoto.note")
            .into(),
    ]);

    let by_category = count_by_category(&reports);
    let category_body = if by_category.is_empty() {
        empty("i18n:stats.byCategory.empty", IconName::DangerTriangle)
    } else {
        Chart::new()
            .kind(ChartKind::HorizontalBars)
            .swatch(Swatch::Red)
            .points(
                by_category
                    .into_iter()
                    .map(|(wire, n)| {
                        ChartPoint::new(format!("i18n:stats.category.{wire}"), n as f64)
                            .display(reports_unit(n, fr))
                    })
                    .collect(),
            )
            .into()
    };

    let delay_body = if delays.is_empty() {
        empty("i18n:stats.delay.empty", IconName::ClockCircle)
    } else {
        delay_chart(&reports, now, days, fr)
    };

    let panels = Grid::new().minColumnWidth(320.0).gap(16.0).children(vec![
        Card::new()
            .title("i18n:stats.byCategory.title")
            .subtitle("i18n:stats.byCategory.subtitle")
            .icon(IconName::DangerTriangle)
            .children(vec![category_body])
            .into(),
        Card::new()
            .title("i18n:stats.delay.title")
            .subtitle("i18n:stats.delay.subtitle")
            .icon(IconName::ClockCircle)
            .children(vec![delay_body])
            .into(),
    ]);

    Surface::new(Page::new().child(Stack::new().gap(16.0).children(vec![
        tiles.into(),
        panels.into(),
        super::recent_reports_card(&ctx.locale),
    ])))
    .with_id(crate::ids::HOST_STATS)
}

/// Dashboard period selector: 30 j / 90 j / 12 mois; anything else falls back to 30.
fn period_days(ctx: &HostContext) -> i64 {
    period_key(ctx.input_u64("periodDays").unwrap_or(30) as i64)
}

fn window_note(days: i64, fr: bool) -> String {
    match (days, fr) {
        (365, true) => "sur 12 mois".into(),
        (365, false) => "over 12 months".into(),
        (d, true) => format!("sur {d} jours"),
        (d, false) => format!("over {d} days"),
    }
}

fn reports_unit(n: usize, fr: bool) -> String {
    match (n, fr) {
        (1, true) => "1 signalement".into(),
        (n, true) => format!("{n} signalements"),
        (1, false) => "1 report".into(),
        (n, false) => format!("{n} reports"),
    }
}

fn average(delays: &[Duration]) -> Option<Duration> {
    let n = i32::try_from(delays.len()).ok().filter(|n| *n > 0)?;
    Some(delays.iter().copied().sum::<Duration>() / n)
}

/// « < 1 h », « 6 h », then days past 48 h (« 3 j »).
pub(crate) fn format_delay(delay: Duration, fr: bool) -> String {
    let hours = delay.num_hours();
    if hours < 1 {
        "< 1 h".into()
    } else if hours < 48 {
        format!("{hours} h")
    } else if fr {
        format!("{} j", delay.num_days())
    } else {
        format!("{} d", delay.num_days())
    }
}

/// Average resolution delay per slice of the period: weeks (S1…) for 30 and 90 days, months
/// for 12 months. Slices without a resolved report show « — ».
fn delay_chart(reports: &[IssueReport], now: DateTime<Utc>, days: i64, fr: bool) -> Component {
    let slices: i64 = match days {
        365 => 12,
        d => (d + 6) / 7,
    };
    let since = now - Duration::days(days);
    let mut buckets = vec![Vec::new(); slices as usize];
    for report in reports {
        let Some(resolved) = report.resolved_at else {
            continue;
        };
        let i = ((report.created_at - since).num_seconds() * slices
            / Duration::days(days).num_seconds())
        .clamp(0, slices - 1);
        buckets[i as usize].push(resolved - report.created_at);
    }
    let points = buckets
        .iter()
        .enumerate()
        .map(|(i, delays)| {
            let label = if days == 365 {
                let mid = since + Duration::days(days) * (2 * i as i32 + 1) / (2 * slices as i32);
                month_initial(mid.month()).to_string()
            } else {
                format!("{}{}", if fr { "S" } else { "W" }, i + 1)
            };
            match average(delays) {
                Some(avg) => ChartPoint::new(label, avg.num_minutes() as f64 / 60.0)
                    .display(format_delay(avg, fr)),
                None => ChartPoint::new(label, 0.0).display("—"),
            }
        })
        .collect();
    Chart::new()
        .kind(ChartKind::Bars)
        .swatch(Swatch::Red)
        .points(points)
        .into()
}

/// Same initials in French and English.
fn month_initial(month: u32) -> &'static str {
    ["J", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"][(month as usize + 11) % 12]
}

fn empty(description: &str, icon: IconName) -> Component {
    EmptyState::new()
        .title("i18n:stats.empty")
        .description(description)
        .icon(icon)
        .into()
}

/// Non-empty categories, most reported first (ties keep the form order; unknown → other).
fn count_by_category(reports: &[IssueReport]) -> Vec<(&'static str, usize)> {
    let mut counts: Vec<(&'static str, usize)> = category::WIRE_VALUES
        .iter()
        .map(|wire| {
            let key = category::category_label_key(wire);
            let n = reports
                .iter()
                .filter(|r| category::category_label_key(&r.category) == key)
                .count();
            (*wire, n)
        })
        .filter(|(_, n)| *n > 0)
        .collect();
    counts.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn report(category: &str) -> IssueReport {
        IssueReport {
            id: Uuid::new_v4(),
            stay_id: Uuid::new_v4(),
            category: category.into(),
            summary: "x".into(),
            details: None,
            created_at: DateTime::<Utc>::UNIX_EPOCH,
            resolved_at: None,
            photo: None,
        }
    }

    #[test]
    fn counts_sorted_desc_and_unknown_folds_into_other() {
        let rows = [
            report("noise"),
            report("appliance"),
            report("appliance"),
            report("legacy-value"),
        ];
        assert_eq!(
            count_by_category(&rows),
            vec![("appliance", 2), ("noise", 1), ("other", 1)]
        );
    }

    #[test]
    fn delays_average_and_format() {
        let d = [Duration::hours(3), Duration::hours(9)];
        assert_eq!(format_delay(average(&d).unwrap(), true), "6 h");
        assert_eq!(format_delay(Duration::minutes(20), true), "< 1 h");
        assert_eq!(format_delay(Duration::hours(80), true), "3 j");
        assert!(average(&[]).is_none());
        assert_eq!(reports_unit(1, true), "1 signalement");
        assert_eq!(reports_unit(2, false), "2 reports");
    }
}
