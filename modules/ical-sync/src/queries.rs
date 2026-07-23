//! Host queries — feed sources + apply parsed ICS bodies (platformFetch path).

use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{load_config, save_config, ModuleConfig};
use crate::ics::{parse_stay_rows, StayImportRow};

const MAX_EVENTS: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedSource {
    pub id: String,
    pub url: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    let mut sources = Vec::new();
    if let Some(url) = config.primary_url() {
        sources.push(FeedSource {
            id: "primary".into(),
            url: url.to_string(),
            provider: guess_provider(url),
        });
    }
    if let Some(url) = config.secondary_url() {
        sources.push(FeedSource {
            id: "secondary".into(),
            url: url.to_string(),
            provider: guess_provider(url),
        });
    }
    Ok(ListSourcesResponse { sources })
}

/// Parses platform-fetched ICS bodies and returns stay rows + updated sync metadata.
#[portaki_sdk::query(name = "applyFeeds")]
pub fn apply_feeds(_ctx: Context, args: ApplyFeedsArgs) -> Result<ApplyFeedsResponse> {
    let guest_lang = if args.guest_lang.trim().is_empty() {
        "fr"
    } else {
        args.guest_lang.trim()
    };

    let mut rows = Vec::new();
    let mut succeeded = 0i32;
    let mut failed = 0i32;
    let mut items_total = 0i32;

    for feed in &args.feeds {
        if feed.ics_body.trim().is_empty() {
            failed += 1;
            continue;
        }
        let remaining = MAX_EVENTS.saturating_sub(rows.len());
        if remaining == 0 {
            break;
        }
        let parsed = parse_stay_rows(&feed.ics_body, guest_lang, remaining);
        if parsed.is_empty() {
            failed += 1;
            continue;
        }
        items_total += parsed.len() as i32;
        succeeded += 1;
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

    let mut config = load_config().unwrap_or_default();
    if !now.is_empty() {
        config.last_sync_at = Some(now);
    }
    config.sync_summary = Some(summary.clone());
    let _ = save_config(&config);

    Ok(ApplyFeedsResponse {
        ok: succeeded > 0,
        succeeded,
        failed,
        items_total,
        summary,
        rows,
        updated_plain_config: config,
    })
}

fn guess_provider(url: &str) -> Option<String> {
    let lower = url.to_ascii_lowercase();
    if lower.contains("airbnb.") {
        Some("airbnb".into())
    } else if lower.contains("booking.com") {
        Some("booking".into())
    } else if lower.contains("abritel.") || lower.contains("vrbo.") {
        Some("vrbo".into())
    } else {
        None
    }
}
