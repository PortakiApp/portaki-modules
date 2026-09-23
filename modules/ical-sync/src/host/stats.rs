//! Property stats strip — `property-stats-card` host surface.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Field, Page, Text};
use portaki_sdk::sdui::surface::Surface;

use chrono::{DateTime, Duration, Utc};
use portaki_sdk::host::time;

use crate::config::load_config;
use crate::sync_state::{load_sync_state, SyncState};

#[portaki_sdk::surface(host, id = "calendar-sync")]
pub fn render_host_stats(ctx: HostContext) -> Surface {
    let config = load_config().unwrap_or_default();
    let connected = config.connected_calendars().len();
    let days = period_days(&ctx);
    let arrivals = time::now()
        .ok()
        .map(|now| arrivals_in_window(&load_sync_state().unwrap_or_default(), now, days));
    let last_sync = config
        .last_sync_at
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "i18n:stats.never".to_string());
    let summary = config
        .sync_summary
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "i18n:stats.emptySummary".to_string());
    let failed = summary_mentions_failures(config.sync_summary.as_deref());

    let mut children: Vec<Component> = vec![
        Field::new()
            .name("connected_calendars")
            .label("i18n:stats.connected")
            .child(
                Text::new()
                    .text(format!("{connected}"))
                    .variant(TextVariant::Body),
            )
            .into(),
        Field::new()
            .name("arrivals")
            .label(format!("i18n:stats.arrivals.{days}"))
            .child(
                Text::new()
                    .text(arrivals.map_or_else(|| "—".to_string(), |n| n.to_string()))
                    .variant(TextVariant::Body),
            )
            .into(),
        Field::new()
            .name("last_sync_at")
            .label("i18n:stats.lastSync")
            .child(Text::new().text(last_sync).variant(TextVariant::Caption))
            .into(),
        Field::new()
            .name("sync_summary")
            .label("i18n:stats.summary")
            .child(Text::new().text(summary).variant(TextVariant::Caption))
            .into(),
    ];

    if failed {
        children.push(
            Text::new()
                .text("i18n:stats.errorsHint")
                .variant(TextVariant::Caption)
                .into(),
        );
    }

    Surface::new(
        Page::new().child(
            Card::new()
                .title("i18n:stats.title")
                .subtitle("i18n:stats.subtitle")
                .icon("calendar")
                .children(children),
        ),
    )
    .with_id(crate::ids::HOST_STATS)
}

/// Dashboard period selector (`input.periodDays`): 30 j / 90 j / 12 mois, default 30.
fn period_days(ctx: &HostContext) -> i64 {
    match ctx.input_u64("periodDays") {
        Some(90) => 90,
        Some(365) => 365,
        _ => 30,
    }
}

/// Synced stays whose check-in falls in the last `days` days.
fn arrivals_in_window(state: &SyncState, now: DateTime<Utc>, days: i64) -> usize {
    let since = now - Duration::days(days);
    state
        .uids
        .values()
        .filter_map(|seen| DateTime::parse_from_rfc3339(&seen.check_in_at).ok())
        .filter(|at| *at >= since && *at <= now)
        .count()
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
    use crate::sync_state::SeenStay;

    #[test]
    fn arrivals_count_only_the_window() {
        let seen = |at: &str| SeenStay {
            check_in_at: at.into(),
            check_out_at: at.into(),
            guest_name: String::new(),
        };
        let state = SyncState {
            uids: [
                ("a".into(), seen("2026-09-20T15:00:00+00:00")),
                ("b".into(), seen("2026-07-01T15:00:00+00:00")),
                ("c".into(), seen("2026-10-01T15:00:00+00:00")),
                ("d".into(), seen("garbage")),
            ]
            .into(),
            last_success_at: None,
        };
        let now = DateTime::parse_from_rfc3339("2026-09-23T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(arrivals_in_window(&state, now, 30), 1);
        assert_eq!(arrivals_in_window(&state, now, 90), 2);
    }
}
