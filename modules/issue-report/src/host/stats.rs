//! Property stats tab — `property-stats-card` host surface (design `tabStats` → « Signalements »).
//!
//! The dashboard passes the selected period as `input.periodDays` (30, 90 or 365).
//! Guests cannot attach photos yet (no guest upload in the SDK), so « Avec photo » stays « — ».

use chrono::Duration;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, EmptyState, Grid, KeyValue, Page, Stack, Stat};
use portaki_sdk::sdui::surface::Surface;

use crate::category;
use crate::entities::IssueReport;
use crate::storage;

#[portaki_sdk::surface(host, id = "issue-stats")]
pub fn render_host_stats(ctx: HostContext) -> Surface {
    let fr = ctx.locale.to_ascii_lowercase().starts_with("fr");
    let days = period_days(&ctx);
    let now = super::host_now();
    let reports = storage::list_since(now - Duration::days(days)).unwrap_or_default();

    let delays: Vec<Duration> = reports
        .iter()
        .filter_map(|r| r.resolved_at.map(|at| at - r.created_at))
        .collect();
    let open_reports: Vec<&IssueReport> =
        reports.iter().filter(|r| r.resolved_at.is_none()).collect();

    let mut resolved = Stat::new()
        .label("i18n:stats.resolved")
        .value(delays.len().to_string());
    if let Some(avg) = average(&delays) {
        let prefix = if fr { "délai moyen" } else { "avg." };
        resolved = resolved.delta(format!("{prefix} {}", format_delay(avg, fr)));
    }
    let mut open = Stat::new()
        .label("i18n:stats.open")
        .value(open_reports.len().to_string());
    if let Some(newest) = open_reports.iter().max_by_key(|r| r.created_at) {
        open = open.delta(newest.summary.clone());
    }

    let tiles = Grid::new().minColumnWidth(170.0).gap(12.0).children(vec![
        Stat::new()
            .label("i18n:stats.reports")
            .value(reports.len().to_string())
            .delta(window_note(days, fr))
            .into(),
        resolved.into(),
        open.into(),
        Stat::new().label("i18n:stats.withPhoto").value("—").into(),
    ]);

    let by_category = count_by_category(&reports);
    let category_body = if by_category.is_empty() {
        empty("i18n:stats.byCategory.empty", "danger-triangle")
    } else {
        rows(
            by_category
                .into_iter()
                .map(|(wire, n)| (format!("i18n:stats.category.{wire}"), reports_unit(n, fr)))
                .collect(),
        )
    };

    let delay_body = match (average(&delays), delays.iter().min(), delays.iter().max()) {
        (Some(avg), Some(min), Some(max)) => rows(vec![
            ("i18n:stats.delay.avg".into(), format_delay(avg, fr)),
            ("i18n:stats.delay.min".into(), format_delay(*min, fr)),
            ("i18n:stats.delay.max".into(), format_delay(*max, fr)),
        ]),
        _ => empty("i18n:stats.delay.empty", "clock-circle"),
    };

    let panels = Grid::new().minColumnWidth(320.0).gap(16.0).children(vec![
        Card::new()
            .title("i18n:stats.byCategory.title")
            .subtitle("i18n:stats.byCategory.subtitle")
            .icon("danger-triangle")
            .children(vec![category_body])
            .into(),
        Card::new()
            .title("i18n:stats.delay.title")
            .subtitle("i18n:stats.delay.subtitle")
            .icon("clock-circle")
            .children(vec![delay_body])
            .into(),
    ]);

    Surface::new(
        Page::new().child(
            Stack::new()
                .gap(16.0)
                .children(vec![tiles.into(), panels.into()]),
        ),
    )
    .with_id(crate::ids::HOST_STATS)
}

/// Dashboard period selector: 30 j / 90 j / 12 mois; anything else falls back to 30.
fn period_days(ctx: &HostContext) -> i64 {
    match ctx.input_u64("periodDays") {
        Some(90) => 90,
        Some(365) => 365,
        _ => 30,
    }
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
fn format_delay(delay: Duration, fr: bool) -> String {
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

fn rows(pairs: Vec<(String, String)>) -> Component {
    Stack::new()
        .gap(12.0)
        .children(
            pairs
                .into_iter()
                .map(|(key, value)| KeyValue::new().key(key).value(value).into())
                .collect(),
        )
        .into()
}

fn empty(description: &str, icon: &str) -> Component {
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
    use chrono::{DateTime, Utc};
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
