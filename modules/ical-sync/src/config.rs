//! Host configuration stored in KV (`config` key).

use portaki_sdk::host;
use portaki_sdk::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const CONFIG_KEY: &str = "config";

/// Soft cap for host SDUI rows (abuse / UI guard). Not a product “max 2”.
pub const CALENDAR_SLOTS: usize = 20;

/// Declared ICS dialect for a feed — drives VEVENT filtering / guest naming.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CalendarFormat {
    Airbnb,
    Booking,
    AbritelVrbo,
    Google,
    #[default]
    Generic,
}

impl CalendarFormat {
    pub const ALL: [CalendarFormat; 5] = [
        CalendarFormat::Airbnb,
        CalendarFormat::Booking,
        CalendarFormat::AbritelVrbo,
        CalendarFormat::Google,
        CalendarFormat::Generic,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            CalendarFormat::Airbnb => "airbnb",
            CalendarFormat::Booking => "booking",
            CalendarFormat::AbritelVrbo => "abritel_vrbo",
            CalendarFormat::Google => "google",
            CalendarFormat::Generic => "generic",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "airbnb" => Some(CalendarFormat::Airbnb),
            "booking" => Some(CalendarFormat::Booking),
            "abritel_vrbo" | "abritel" | "vrbo" | "homeaway" => Some(CalendarFormat::AbritelVrbo),
            "google" => Some(CalendarFormat::Google),
            "generic" | "other" | "" => Some(CalendarFormat::Generic),
            _ => None,
        }
    }

    /// Best-effort guess from export URL host (migration / legacy writes).
    pub fn detect_from_url(url: &str) -> Option<Self> {
        let lower = url.to_ascii_lowercase();
        if lower.contains("airbnb.") {
            Some(CalendarFormat::Airbnb)
        } else if lower.contains("booking.com") {
            Some(CalendarFormat::Booking)
        } else if lower.contains("abritel.")
            || lower.contains("vrbo.")
            || lower.contains("homeaway.")
        {
            Some(CalendarFormat::AbritelVrbo)
        } else if lower.contains("calendar.google.com")
            || lower.contains("google.com/calendar")
            || lower.contains("googleapis.com/calendar")
        {
            Some(CalendarFormat::Google)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalendarFeed {
    pub id: String,
    #[serde(default)]
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Platform ICS dialect. Always persisted after migrate / save.
    #[serde(default)]
    pub format: CalendarFormat,
}

impl CalendarFeed {
    pub fn trimmed_url(&self) -> Option<&str> {
        trim_url(&self.url)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    /// Sole source of truth for connected calendar feeds.
    #[serde(default)]
    pub calendars: Vec<CalendarFeed>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_sync_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync_summary: Option<String>,
}

impl ModuleConfig {
    pub fn connected_calendars(&self) -> Vec<&CalendarFeed> {
        self.calendars
            .iter()
            .filter(|c| c.trimmed_url().is_some())
            .collect()
    }

    pub fn has_any_feed(&self) -> bool {
        !self.connected_calendars().is_empty()
    }

    pub fn format_for_id(&self, id: &str) -> CalendarFormat {
        self.calendars
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.format)
            .unwrap_or(CalendarFormat::Generic)
    }
}

fn trim_url(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Wire format that accepts the calendars list plus legacy primary/secondary / feeds_json.
/// Legacy keys are load-only — never written back.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct RawConfig {
    calendars: Vec<RawCalendarFeed>,
    ical_url_primary: String,
    ical_url_secondary: String,
    feeds_json: String,
    last_sync_at: Option<String>,
    sync_summary: Option<String>,
}

/// Load-only feed row — `format` absent means migrate from URL when safe.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct RawCalendarFeed {
    id: String,
    url: String,
    label: Option<String>,
    format: Option<CalendarFormat>,
}

fn migrate_raw(raw: RawConfig) -> ModuleConfig {
    let mut calendars = raw
        .calendars
        .into_iter()
        .enumerate()
        .filter_map(|(index, feed)| migrate_raw_feed(index, feed))
        .collect::<Vec<_>>();

    if calendars.is_empty() {
        calendars = calendars_from_legacy_urls(&raw.ical_url_primary, &raw.ical_url_secondary);
    }
    if calendars.is_empty() {
        calendars = calendars_from_feeds_json(&raw.feeds_json);
    }

    ModuleConfig {
        calendars,
        last_sync_at: nonempty_opt(raw.last_sync_at),
        sync_summary: nonempty_opt(raw.sync_summary),
    }
}

fn migrate_raw_feed(index: usize, mut feed: RawCalendarFeed) -> Option<CalendarFeed> {
    let url = feed.url.trim().to_string();
    if url.is_empty() {
        return None;
    }
    if feed.id.trim().is_empty() {
        feed.id = format!("cal-{}", index + 1);
    }
    let label = feed.label.take().and_then(|label| {
        let trimmed = label.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });
    let format = feed.format.unwrap_or_else(|| {
        CalendarFormat::detect_from_url(&url).unwrap_or(CalendarFormat::Generic)
    });
    Some(CalendarFeed {
        id: feed.id.trim().to_string(),
        url,
        label,
        format,
    })
}

fn calendars_from_legacy_urls(primary: &str, secondary: &str) -> Vec<CalendarFeed> {
    let mut out = Vec::new();
    if let Some(url) = trim_url(primary) {
        out.push(CalendarFeed {
            id: "cal-1".into(),
            url: url.to_string(),
            label: None,
            format: CalendarFormat::detect_from_url(url).unwrap_or(CalendarFormat::Generic),
        });
    }
    if let Some(url) = trim_url(secondary) {
        out.push(CalendarFeed {
            id: "cal-2".into(),
            url: url.to_string(),
            label: None,
            format: CalendarFormat::detect_from_url(url).unwrap_or(CalendarFormat::Generic),
        });
    }
    out
}

fn calendars_from_feeds_json(raw: &str) -> Vec<CalendarFeed> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let Ok(values) = serde_json::from_str::<Vec<Value>>(trimmed) else {
        return Vec::new();
    };
    values
        .into_iter()
        .enumerate()
        .filter_map(|(index, value)| {
            let url = value
                .get("url")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())?
                .to_string();
            let id = value
                .get("id")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| format!("cal-{}", index + 1));
            let label = value
                .get("label")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string);
            let format = value
                .get("format")
                .and_then(|v| v.as_str())
                .and_then(CalendarFormat::parse)
                .or_else(|| CalendarFormat::detect_from_url(&url))
                .unwrap_or(CalendarFormat::Generic);
            Some(CalendarFeed {
                id,
                url,
                label,
                format,
            })
        })
        .collect()
}

fn nonempty_opt(value: Option<String>) -> Option<String> {
    value.and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub fn load_config() -> Result<ModuleConfig> {
    let Some(bytes) = host::kv::get(CONFIG_KEY)? else {
        return Ok(ModuleConfig::default());
    };
    let raw: RawConfig = serde_json::from_slice(&bytes).map_err(|error| {
        portaki_sdk::PortakiError::Storage(format!("invalid config JSON: {error}"))
    })?;
    Ok(migrate_raw(raw))
}

pub fn save_config(config: &ModuleConfig) -> Result<()> {
    let mut config = config.clone();
    // Drop empty rows before persist. Never write legacy primary/secondary keys.
    config.calendars.retain(|c| c.trimmed_url().is_some());
    let bytes = serde_json::to_vec(&config).map_err(|error| {
        portaki_sdk::PortakiError::Storage(format!("config serialize: {error}"))
    })?;
    host::kv::set(CONFIG_KEY, &bytes, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_primary_secondary_urls() {
        let cfg = migrate_raw(RawConfig {
            ical_url_primary: " https://a.ics ".into(),
            ical_url_secondary: "https://b.ics".into(),
            ..RawConfig::default()
        });
        assert_eq!(cfg.calendars.len(), 2);
        assert_eq!(cfg.calendars[0].url, "https://a.ics");
        assert_eq!(cfg.calendars[1].url, "https://b.ics");
        assert_eq!(cfg.calendars[0].format, CalendarFormat::Generic);
        let json = serde_json::to_value(&cfg).expect("serialize");
        assert!(json.get("ical_url_primary").is_none());
        assert!(json.get("ical_url_secondary").is_none());
    }

    #[test]
    fn migrates_feeds_json() {
        let cfg = migrate_raw(RawConfig {
            feeds_json:
                r#"[{"url":"https://x.ics"},{"url":"https://y.ics"},{"url":"https://z.ics"}]"#
                    .into(),
            ..RawConfig::default()
        });
        assert_eq!(cfg.calendars.len(), 3);
        assert_eq!(cfg.calendars[0].url, "https://x.ics");
        let json = serde_json::to_value(&cfg).expect("serialize");
        assert!(json.get("feeds_json").is_none());
    }

    #[test]
    fn calendars_list_wins_over_legacy() {
        let cfg = migrate_raw(RawConfig {
            calendars: vec![RawCalendarFeed {
                id: "c1".into(),
                url: "https://only.ics".into(),
                label: Some("Airbnb".into()),
                format: Some(CalendarFormat::Airbnb),
            }],
            ical_url_primary: "https://legacy.ics".into(),
            ical_url_secondary: "https://legacy2.ics".into(),
            ..RawConfig::default()
        });
        assert_eq!(cfg.calendars.len(), 1);
        assert_eq!(cfg.calendars[0].url, "https://only.ics");
        assert_eq!(cfg.calendars[0].format, CalendarFormat::Airbnb);
    }

    #[test]
    fn missing_format_detects_from_airbnb_url() {
        let cfg = migrate_raw(RawConfig {
            calendars: vec![RawCalendarFeed {
                id: "c1".into(),
                url: "https://www.airbnb.com/calendar/ical/1.ics".into(),
                label: None,
                format: None,
            }],
            ..RawConfig::default()
        });
        assert_eq!(cfg.calendars[0].format, CalendarFormat::Airbnb);
    }

    #[test]
    fn explicit_generic_is_kept_even_on_airbnb_url() {
        let cfg = migrate_raw(RawConfig {
            calendars: vec![RawCalendarFeed {
                id: "c1".into(),
                url: "https://www.airbnb.com/calendar/ical/1.ics".into(),
                label: None,
                format: Some(CalendarFormat::Generic),
            }],
            ..RawConfig::default()
        });
        assert_eq!(cfg.calendars[0].format, CalendarFormat::Generic);
    }

    #[test]
    fn empty_urls_are_ignored() {
        let config = ModuleConfig {
            calendars: vec![
                CalendarFeed {
                    id: "a".into(),
                    url: "  ".into(),
                    label: None,
                    format: CalendarFormat::Generic,
                },
                CalendarFeed {
                    id: "b".into(),
                    url: "https://example.com/a.ics".into(),
                    label: None,
                    format: CalendarFormat::Booking,
                },
            ],
            ..Default::default()
        };
        assert_eq!(config.connected_calendars().len(), 1);
        assert!(config.has_any_feed());
        assert_eq!(config.format_for_id("b"), CalendarFormat::Booking);
    }

    #[test]
    fn format_wire_roundtrip() {
        for format in CalendarFormat::ALL {
            assert_eq!(CalendarFormat::parse(format.as_str()), Some(format));
        }
        assert_eq!(
            CalendarFormat::parse("vrbo"),
            Some(CalendarFormat::AbritelVrbo)
        );
    }
}
