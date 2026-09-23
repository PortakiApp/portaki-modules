//! Property stats tab — `property-stats-card` host surface (design `tabStats` → « Signalements »).
//!
//! The surface contract passes no period, so the window is the design default: 30 days.
//! Open/resolved and photos are not persisted yet — those tiles and the resolution-delay
//! panel stay honest (« — », empty state) instead of showing made-up numbers.

use chrono::Duration;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, EmptyState, Grid, KeyValue, Page, Stack, Stat};
use portaki_sdk::sdui::surface::Surface;

use crate::category;
use crate::entities::IssueReport;
use crate::storage;

const WINDOW_DAYS: i64 = 30;
const NONE: &str = "—";

#[portaki_sdk::surface(host, id = "issue-stats")]
pub fn render_host_stats(_ctx: HostContext) -> Surface {
    let since = super::host_now() - Duration::days(WINDOW_DAYS);
    let reports = storage::list_since(since).unwrap_or_default();

    let tiles = Grid::new().minColumnWidth(170.0).gap(12.0).children(vec![
        Stat::new()
            .label("i18n:stats.reports")
            .value(reports.len().to_string())
            .delta("i18n:stats.window30")
            .into(),
        Stat::new().label("i18n:stats.resolved").value(NONE).into(),
        Stat::new().label("i18n:stats.open").value(NONE).into(),
        Stat::new().label("i18n:stats.withPhoto").value(NONE).into(),
    ]);

    let by_category = count_by_category(&reports);
    let category_body: Component = if by_category.is_empty() {
        EmptyState::new()
            .title("i18n:host.main.emptyRecent")
            .description("i18n:stats.byCategory.empty")
            .icon("danger-triangle")
            .into()
    } else {
        Stack::new()
            .gap(12.0)
            .children(
                by_category
                    .into_iter()
                    .map(|(wire, n)| {
                        KeyValue::new()
                            .key(format!("i18n:{}", category::category_label_key(wire)))
                            .value(n.to_string())
                            .into()
                    })
                    .collect(),
            )
            .into()
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
            .children(vec![EmptyState::new()
                .title("i18n:stats.delay.empty")
                .description("i18n:stats.delay.empty.help")
                .icon("clock-circle")
                .into()])
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

/// Non-empty categories, most reported first (ties keep the form order).
fn count_by_category(reports: &[IssueReport]) -> Vec<(&'static str, usize)> {
    let mut counts: Vec<(&'static str, usize)> = category::WIRE_VALUES
        .iter()
        .map(|wire| {
            let n = reports
                .iter()
                .filter(|r| {
                    category::category_label_key(&r.category) == category::category_label_key(wire)
                })
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
}
