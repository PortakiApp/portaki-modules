//! Module commands — configuration persistence.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{
    load_config, save_config, CalendarFeed, CalendarFormat, ModuleConfig, CALENDAR_SLOTS,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalendarInput {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub label: String,
    /// Platform ICS dialect (`airbnb`, `booking`, `abritel_vrbo`, `google`, `generic`).
    #[serde(default)]
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateConfigArgs {
    /// Dynamic list from host StepList (`calendars.{i}.url` / `.label` / `.id` / `.format`).
    #[serde(default)]
    pub calendars: Vec<CalendarInput>,
    /// Legacy flat fields (pre multi-calendar) — accepted on write, converted to `calendars`.
    #[serde(default)]
    pub ical_url_primary: String,
    #[serde(default)]
    pub ical_url_secondary: String,
}

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(_ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    let existing = load_config().unwrap_or_default();
    let calendars = calendars_from_args(&args);
    save_config(&ModuleConfig {
        calendars,
        last_sync_at: existing.last_sync_at,
        sync_summary: existing.sync_summary,
    })
}

fn calendars_from_args(args: &UpdateConfigArgs) -> Vec<CalendarFeed> {
    if !args.calendars.is_empty() {
        return args
            .calendars
            .iter()
            .take(CALENDAR_SLOTS)
            .enumerate()
            .filter_map(|(index, input)| {
                let url = input.url.trim();
                if url.is_empty() {
                    return None;
                }
                let id = {
                    let trimmed = input.id.trim();
                    if trimmed.is_empty() {
                        format!("cal-{}", index + 1)
                    } else {
                        trimmed.to_string()
                    }
                };
                let label = {
                    let trimmed = input.label.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                };
                let format = resolve_format(&input.format, url);
                Some(CalendarFeed {
                    id,
                    url: url.to_string(),
                    label,
                    format,
                })
            })
            .collect();
    }

    // Legacy primary / secondary fallback (not persisted as those keys).
    let mut out = Vec::new();
    let primary = args.ical_url_primary.trim();
    if !primary.is_empty() {
        out.push(CalendarFeed {
            id: "cal-1".into(),
            url: primary.to_string(),
            label: None,
            format: CalendarFormat::detect_from_url(primary).unwrap_or(CalendarFormat::Generic),
        });
    }
    let secondary = args.ical_url_secondary.trim();
    if !secondary.is_empty() {
        out.push(CalendarFeed {
            id: "cal-2".into(),
            url: secondary.to_string(),
            label: None,
            format: CalendarFormat::detect_from_url(secondary).unwrap_or(CalendarFormat::Generic),
        });
    }
    out
}

fn resolve_format(raw: &str, url: &str) -> CalendarFormat {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return CalendarFormat::detect_from_url(url).unwrap_or(CalendarFormat::Generic);
    }
    CalendarFormat::parse(trimmed)
        .or_else(|| CalendarFormat::detect_from_url(url))
        .unwrap_or(CalendarFormat::Generic)
}
