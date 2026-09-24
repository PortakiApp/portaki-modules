//! Property statistics « Objets trouvés » — the tile (`statsSummary`) and the
//! `property-stats-detail` surface: counters and the declared items, no chart.
//!
//! An item still `to_collect` after [`NO_ANSWER_DAYS`] reads « Sans réponse ». A row opens the
//! dashboard detail modal (`host.surface.overlay`), which renders this surface again with the
//! row's `itemId`: the item, its follow-up and « Marquer comme rendu » (`updateStatus`).

use chrono::{DateTime, Duration, Utc};
use portaki_sdk::contracts::stats::{self, AttentionLevel, StatsSummary, StatsSummaryArgs};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{
    Button, Card, ChecklistItem, EmptyState, FeedItem, Grid, KeyValue, Page, Stack, Stat, Text,
};
use portaki_sdk::sdui::surface::Surface;
use portaki_sdk::sdui::{FeedStatus, NavigateTarget};
use serde_json::json;
use uuid::Uuid;

use crate::commands::UpdateStatusArgs;
use crate::entities::LostFoundReport;
use crate::{i18n, status, storage};

use super::status_ui::{format_short_date, report_title, source_label};

const NO_ANSWER_DAYS: i64 = 14;

/// Where a declared item stands.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Waiting,
    NoAnswer,
    Returned,
}

fn stage(report: &LostFoundReport, now: DateTime<Utc>) -> Stage {
    if report.status != status::DEFAULT {
        Stage::Returned
    } else if now - report.created_at >= Duration::days(NO_ANSWER_DAYS) {
        Stage::NoAnswer
    } else {
        Stage::Waiting
    }
}

fn count(reports: &[LostFoundReport], now: DateTime<Utc>, wanted: Stage) -> usize {
    reports.iter().filter(|r| stage(r, now) == wanted).count()
}

/// Bundle suffix of a period: 30, 90 or 365 (anything else reads as 30).
fn period_key(days: i64) -> i64 {
    match days {
        90 | 365 => days,
        _ => 30,
    }
}

/// Tile: items declared over the period, those awaiting the guest as attention.
#[portaki_sdk::query(name = "statsSummary")]
pub fn stats_summary(_ctx: Context, args: StatsSummaryArgs) -> Result<StatsSummary> {
    let days = period_key(i64::from(args.period));
    let now = time::now()?;
    let reports = storage::list_since(now - Duration::days(days))?;
    let tile = stats::summary(
        reports.len().to_string(),
        i18n::text(&format!("stats.tile.{days}"), &[]),
    );
    let waiting = count(&reports, now, Stage::Waiting) + count(&reports, now, Stage::NoAnswer);
    if waiting == 0 {
        return Ok(tile);
    }
    let waiting = waiting.to_string();
    Ok(tile.attention(
        AttentionLevel::Warning,
        i18n::text("stats.tile.waiting", &[("count", &waiting)]),
    ))
}

#[portaki_sdk::surface(
    host,
    id = "lost-stats",
    placement = HostPlacement::PropertyStatsCard,
    placement = HostPlacement::PropertyStatsDetail,
    label_key = "catalog.host.lost-stats",
    icon = IconName::Search
)]
pub fn render_host_stats(ctx: HostContext) -> Surface {
    let locale = ctx.locale.as_str();
    let days = period_key(ctx.input_u64("periodDays").unwrap_or(30) as i64);
    let now = time::now().unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    if let Some(report) = ctx
        .input_str("itemId")
        .and_then(|id| Uuid::parse_str(id).ok())
        .and_then(|id| storage::find_by_id(id).ok().flatten())
    {
        return Surface::new(item_detail(&report, now, locale)).with_id(crate::ids::HOST_STATS);
    }
    let reports = storage::list_since(now - Duration::days(days)).unwrap_or_default();

    let stat = |label: &str, value: usize, note: String| -> Component {
        Stat::new()
            .label(format!("i18n:stats.{label}"))
            .value(value.to_string())
            .delta(note)
            .into()
    };
    let tiles = Grid::new().minColumnWidth(170.0).gap(12.0).children(vec![
        stat(
            "declared",
            reports.len(),
            t!(&format!("stats.window.{days}")).unwrap_or_default(),
        ),
        stat(
            "returned",
            count(&reports, now, Stage::Returned),
            "i18n:stats.returned.note".into(),
        ),
        stat(
            "waiting",
            count(&reports, now, Stage::Waiting),
            "i18n:stats.waiting.note".into(),
        ),
        stat(
            "noAnswer",
            count(&reports, now, Stage::NoAnswer),
            "i18n:stats.noAnswer.note".into(),
        ),
    ]);

    let feed: Component = if reports.is_empty() {
        EmptyState::new()
            .title("i18n:host.main.emptyRecent")
            .description("i18n:host.main.emptyRecent.help")
            .icon(IconName::Search)
            .into()
    } else {
        Stack::new()
            .gap(0.0)
            .children(
                reports
                    .iter()
                    .map(|report| item_row(report, now, locale))
                    .collect(),
            )
            .into()
    };

    Surface::new(Page::new().child(Stack::new().gap(16.0).children(vec![
                tiles.into(),
                Card::new()
                    .title("i18n:host.main.recentTitle")
                    .subtitle("i18n:host.main.recentHelp")
                    .child(feed)
                    .into(),
            ])))
    .with_id(crate::ids::HOST_STATS)
}

/// Status key and tone of an item.
fn status_of(report: &LostFoundReport, now: DateTime<Utc>) -> (&'static str, Tone) {
    match stage(report, now) {
        Stage::Waiting => ("stats.status.waiting", Tone::Warning),
        Stage::NoAnswer => ("stats.status.noAnswer", Tone::Neutral),
        Stage::Returned => ("stats.status.returned", Tone::Success),
    }
}

fn item_row(report: &LostFoundReport, now: DateTime<Utc>, locale: &str) -> Component {
    let (status, tone) = status_of(report, now);
    let title = report_title(report);
    // The modal header is plain text: resolved here, in the host's language.
    let open_detail = Action::emit(
        crate::ids::HOST_SURFACE_OVERLAY,
        Some(json!({
            "input": { "itemId": report.id },
            "icon": "search",
            "kicker": t!("stats.detail.kicker", source = source_label(&report.kind, locale))
                .unwrap_or_default(),
            "title": title,
            "status": { "label": t!(status).unwrap_or_default(), "tone": tone },
        })),
    );
    FeedItem::new()
        .title(title)
        .meta(source_label(&report.kind, locale))
        .date(
            t!(
                "stats.row.declaredAt",
                date = format_short_date(report.created_at, locale)
            )
            .unwrap_or_default(),
        )
        .dotTone(tone)
        .status(FeedStatus::new(format!("i18n:{status}"), tone))
        .action(open_detail)
        .into()
}

/// Body of the detail modal: the item, its follow-up, the guest's options and the actions.
fn item_detail(report: &LostFoundReport, now: DateTime<Utc>, locale: &str) -> Component {
    let declared = format_short_date(report.created_at, locale);
    let returned = stage(report, now) == Stage::Returned;
    let mut children: Vec<Component> = vec![
        KeyValue::new()
            .key("i18n:stats.detail.source")
            .value(source_label(&report.kind, locale))
            .into(),
        KeyValue::new()
            .key("i18n:stats.detail.date")
            .value(declared.clone())
            .into(),
    ];
    if let Some(details) = &report.details {
        children.push(Text::new().text(details.clone()).into());
    }
    children.extend([
        Text::new()
            .text("i18n:stats.detail.followUp")
            .variant(TextVariant::Title)
            .into(),
        ChecklistItem::new()
            .label(t!("stats.detail.declared", date = declared).unwrap_or_default())
            .checked(true)
            .into(),
        ChecklistItem::new()
            .label("i18n:stats.detail.returned")
            .checked(returned)
            .into(),
        Text::new()
            .text("i18n:stats.detail.note")
            .variant(TextVariant::Caption)
            .into(),
        Button::new()
            .label("i18n:stats.detail.openStay")
            .variant(ButtonVariant::Outline)
            .action(Action::navigate(
                NavigateTarget::path(format!("/stays/{}", report.stay_id)),
                None,
            ))
            .into(),
        Button::new()
            .label("i18n:stats.detail.message")
            .variant(ButtonVariant::Outline)
            .action(Action::navigate(NavigateTarget::path("/messages"), None))
            .into(),
    ]);
    if !returned {
        children.push(
            Button::new()
                .label("i18n:stats.detail.markReturned")
                .action(crate::ids::module_id().command(
                    crate::ids::UPDATE_STATUS,
                    UpdateStatusArgs {
                        report_id: report.id,
                        status: "returned".into(),
                    },
                ))
                .into(),
        );
    }
    Stack::new().gap(12.0).children(children).into()
}
