//! Host queries — feed sources + apply parsed ICS bodies (platformFetch path).

use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{load_config, save_config, CalendarFormat, ModuleConfig};
use crate::email_send;
use crate::ics::{parse_stay_rows, FeedParseContext, StayImportRow};
use crate::sync_state::{self, SyncDiff};

const MAX_EVENTS: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedSource {
    pub id: String,
    pub url: String,
    /// Declared calendar format (`airbnb`, `booking`, …) for the platform / apply path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSourcesResponse {
    pub sources: Vec<FeedSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedBody {
    pub id: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub ics_body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApplyFeedsArgs {
    #[serde(default)]
    pub guest_lang: String,
    #[serde(default)]
    pub feeds: Vec<FeedBody>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyFeedsResponse {
    pub ok: bool,
    pub succeeded: i32,
    pub failed: i32,
    pub items_total: i32,
    pub summary: String,
    pub rows: Vec<StayImportRow>,
    pub updated_plain_config: ModuleConfig,
}

#[portaki_sdk::query(name = "getConfig")]
pub fn get_config(_ctx: Context) -> Result<ModuleConfig> {
    load_config()
}

/// Returns HTTPS .ics URLs for the platform to fetch (`hostScheduledSync.sourcesQuery`).
#[portaki_sdk::query(name = "listSources")]
pub fn list_sources(_ctx: Context) -> Result<ListSourcesResponse> {
    let config = load_config().unwrap_or_default();
    let sources = config
        .connected_calendars()
        .into_iter()
        .filter_map(|feed| {
            let url = feed.trimmed_url()?;
            Some(FeedSource {
                id: feed.id.clone(),
                url: url.to_string(),
                provider: Some(feed.format.as_str().to_string()),
            })
        })
        .collect();
    Ok(ListSourcesResponse { sources })
}

/// Parses platform-fetched ICS bodies and returns stay rows + updated sync metadata.
///
/// Also triggers module-owned host emails (`sync-failed`, `stay-imported`, `sync-summary`)
/// via `host::email::send`.
#[portaki_sdk::query(name = "applyFeeds")]
pub fn apply_feeds(ctx: Context, args: ApplyFeedsArgs) -> Result<ApplyFeedsResponse> {
    let guest_lang = if args.guest_lang.trim().is_empty() {
        "fr"
    } else {
        args.guest_lang.trim()
    };

    let config = load_config().unwrap_or_default();
    let previous_state = sync_state::load_sync_state().unwrap_or_default();
    let previous_last_success = previous_state
        .last_success_at
        .clone()
        .or_else(|| config.last_sync_at.clone());

    let mut rows = Vec::new();
    let mut succeeded = 0i32;
    let mut failed = 0i32;
    let mut items_total = 0i32;
    let mut failed_feeds: Vec<(String, CalendarFormat, Option<String>)> = Vec::new();
    // First successful feed format — used as source hint for single-stay emails.
    let mut primary_source: Option<(CalendarFormat, Option<String>)> = None;

    for feed in &args.feeds {
        let parse_context = resolve_parse_context(feed, &config);
        let format = parse_context.format;
        let feed_label = config.feed_for_id(&feed.id).and_then(|c| c.label.clone());

        if feed.ics_body.trim().is_empty() {
            failed += 1;
            failed_feeds.push((feed.id.clone(), format, feed_label));
            continue;
        }
        let remaining = MAX_EVENTS.saturating_sub(rows.len());
        if remaining == 0 {
            break;
        }
        let parsed = parse_stay_rows(&feed.ics_body, guest_lang, remaining, &parse_context);
        // Non-empty body = fetch ok. Zero stays can mean “only blocks” — not a failure.
        items_total += parsed.len() as i32;
        succeeded += 1;
        if primary_source.is_none() {
            primary_source = Some((format, feed_label));
        }
        rows.extend(parsed);
    }

    let now = time::now()
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|_| String::new());
    let summary = format!(
        "{} stay(s) · {} feed(s) ok · {} feed(s) failed",
        rows.len(),
        succeeded,
        failed
    );

    let mut config = config;
    if !now.is_empty() {
        config.last_sync_at = Some(now.clone());
    }
    config.sync_summary = Some(summary.clone());
    let _ = save_config(&config);

    let diff = if succeeded > 0 {
        sync_state::diff_rows(&previous_state, &rows)
    } else {
        SyncDiff::default()
    };

    if succeeded > 0 {
        let next = sync_state::next_state(&rows, Some(now.clone()).filter(|s| !s.is_empty()));
        let _ = sync_state::save_sync_state(&next);
    }

    dispatch_sync_emails(
        &ctx,
        &failed_feeds,
        previous_last_success.as_deref(),
        &now,
        &diff,
        primary_source.as_ref(),
    );

    Ok(ApplyFeedsResponse {
        ok: succeeded > 0 || (failed == 0 && args.feeds.is_empty()),
        succeeded,
        failed,
        items_total,
        summary,
        rows,
        updated_plain_config: config,
    })
}

fn dispatch_sync_emails(
    ctx: &Context,
    failed_feeds: &[(String, CalendarFormat, Option<String>)],
    previous_last_success: Option<&str>,
    now: &str,
    diff: &SyncDiff,
    primary_source: Option<&(CalendarFormat, Option<String>)>,
) {
    let property_id = ctx.property_id;
    let property_name = ctx.property.name.as_str();
    let day_key = day_key_from_now(now);

    for (feed_id, format, label) in failed_feeds {
        let source = email_send::source_label(*format, label.as_deref());
        let _ = email_send::notify_sync_failed(
            property_id,
            property_name,
            feed_id,
            &source,
            previous_last_success,
            &day_key,
        );
    }

    if diff.is_empty() {
        return;
    }

    let (format, label) = primary_source
        .map(|(f, l)| (*f, l.clone()))
        .unwrap_or((CalendarFormat::Generic, None));
    let source = email_send::source_label(format, label.as_deref());

    // One new stay alone → focused “complete stay” mail; otherwise batch digest.
    if diff.new_rows.len() == 1 && diff.updated_rows.is_empty() {
        let row = &diff.new_rows[0];
        if row.guest_email.as_deref().unwrap_or("").trim().is_empty() {
            let _ = email_send::notify_stay_imported(property_id, property_name, &source, row);
            return;
        }
    }

    let sync_email_id = if now.is_empty() {
        format!("sync-summary-{day_key}")
    } else {
        format!("sync-summary-{now}")
    };
    let _ = email_send::notify_sync_summary(property_id, &sync_email_id, now, diff);
}

fn day_key_from_now(now: &str) -> String {
    if now.len() >= 10 {
        now[..10].to_string()
    } else {
        "unknown".into()
    }
}

/// Feed shape comes from the platform payload when present, else from config.
/// The platform declaration is always read from config — the fetch payload has
/// no reason to carry it and the feed URL is never consulted here.
fn resolve_parse_context(feed: &FeedBody, config: &ModuleConfig) -> FeedParseContext {
    let (declared_channel, declared_channel_signal) = config.channel_for_id(&feed.id);
    FeedParseContext {
        format: resolve_feed_format(feed, config),
        declared_channel,
        declared_channel_signal,
    }
}

fn resolve_feed_format(feed: &FeedBody, config: &ModuleConfig) -> CalendarFormat {
    feed.provider
        .as_deref()
        .and_then(CalendarFormat::parse)
        .unwrap_or_else(|| config.format_for_id(&feed.id))
}
