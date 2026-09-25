//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use portaki_sdk::contracts::booking_channel::{BookingChannel, ChannelSignal};
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::channel;

/// Soft cap for host SDUI rows (abuse / UI guard). Not a product “max 2”.
pub const CALENDAR_SLOTS: usize = 20;

/// Declared ICS dialect for a feed — drives VEVENT filtering / guest naming.
///
/// This is the feed **shape**, not the seller: `Google` is a mirror / transport
/// and `Generic` means unidentified. Who sold the stay lives on
/// [`CalendarFeed::channel`] as a [`BookingChannel`].
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
    pub url: String,
    pub label: Option<String>,
    /// Platform ICS dialect, deduced when the row does not carry one.
    pub format: CalendarFormat,
    /// Selling platform for this feed. `Unknown` = not declared.
    pub channel: BookingChannel,
    /// Provenance of `channel` — only `HostOverride`, `FeedUrlHost`, or `None`.
    /// Import weighs an explicit choice above a URL prefill.
    pub channel_signal: ChannelSignal,
}

impl CalendarFeed {
    pub fn trimmed_url(&self) -> Option<&str> {
        trim_url(&self.url)
    }
}

/// Resolves the persisted platform declaration for a feed.
///
/// `raw` is the host selector submission. Empty / unparseable / `unknown` falls
/// back to the URL prefill, which is why the provenance is carried alongside.
pub fn resolve_channel(raw: &str, url: &str) -> (BookingChannel, ChannelSignal) {
    if let Some(declared) = BookingChannel::parse(raw).filter(|c| c.is_identified()) {
        return (declared, ChannelSignal::HostOverride);
    }
    match channel::prefill_from_url(url) {
        Some(prefilled) => (prefilled, ChannelSignal::FeedUrlHost),
        None => (BookingChannel::Unknown, ChannelSignal::None),
    }
}

/// Deduces the feed shape — the host picks a platform, not a format. A stored explicit
/// `format` (rows saved before the platform held the config) is still honoured; otherwise
/// the feed URL wins (a `google.com` calendar is a Google mirror whoever sold the stay),
/// then the chosen platform implies its own export shape.
fn resolve_format(raw: &str, url: &str, channel: BookingChannel) -> CalendarFormat {
    CalendarFormat::parse(raw)
        .filter(|_| !raw.trim().is_empty())
        .or_else(|| CalendarFormat::detect_from_url(url))
        .or_else(|| crate::channel::format_from_channel(channel))
        .unwrap_or(CalendarFormat::Generic)
}

/// One calendar row as stored: what the host form sends (`calendars.N.id|channel|label|url`,
/// all strings — a removed row comes back with every field blank), or a row the module saved
/// itself before the platform held the config (plus `format` and `channel_signal`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct CalendarRow {
    pub id: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub format: String,
    pub channel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_signal: Option<ChannelSignal>,
}

/// The host config, held by the platform (`#[portaki_sdk::config]`), which also takes
/// `updateConfig`. Sync status lives in the `sync_state` KV, not here.
///
/// Only recommended: the platform warns while no row was ever saved. A saved list keeps its
/// blank rows (the form never drops one), so it no longer warns once saved — even empty.
#[portaki_sdk::config(legacy = legacy)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    #[field(structured, recommended, label = "host.calendars.label")]
    pub calendars: Vec<CalendarRow>,
}

/// The calendars of this install, normalized: blank rows dropped, format and channel resolved.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    pub calendars: Vec<CalendarFeed>,
}

impl ModuleConfig {
    /// The calendars of this install, from the platform (or the old KV blob through [`legacy`]).
    pub fn load(ctx: &Context) -> Result<Self> {
        Ok(Self {
            calendars: feeds(&Config::load(ctx)?.calendars),
        })
    }

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
        self.feed_for_id(id)
            .map(|c| c.format)
            .unwrap_or(CalendarFormat::Generic)
    }

    /// Persisted platform declaration for a feed, with its provenance.
    pub fn channel_for_id(&self, id: &str) -> (BookingChannel, ChannelSignal) {
        self.feed_for_id(id)
            .map(|c| (c.channel, c.channel_signal))
            .unwrap_or_default()
    }

    pub fn feed_for_id(&self, id: &str) -> Option<&CalendarFeed> {
        self.calendars.iter().find(|c| c.id == id)
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

fn feeds(rows: &[CalendarRow]) -> Vec<CalendarFeed> {
    rows.iter()
        .take(CALENDAR_SLOTS)
        .enumerate()
        .filter_map(|(index, row)| feed(index, row))
        .collect()
}

fn feed(index: usize, row: &CalendarRow) -> Option<CalendarFeed> {
    let url = trim_url(&row.url)?;
    let id = match row.id.trim() {
        "" => format!("cal-{}", index + 1),
        id => id.to_string(),
    };
    let label = row
        .label
        .as_deref()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string);
    // A stored provenance stands with its declared channel; the form sends none, so a
    // host's pick reads as an override and a blank one as the URL prefill.
    let declared = BookingChannel::parse(&row.channel).filter(|c| c.is_identified());
    let (channel, channel_signal) = match (declared, row.channel_signal) {
        (Some(declared), Some(signal)) => (declared, signal),
        _ => resolve_channel(&row.channel, url),
    };
    Some(CalendarFeed {
        id,
        url: url.to_string(),
        label,
        format: resolve_format(&row.format, url, channel),
        channel,
        channel_signal,
    })
}

/// The KV blob the module wrote itself: the calendars list, else the older primary /
/// secondary URLs (ids `cal-1` / `cal-2`, as they were synced under), else `feeds_json`.
fn legacy(mut old: Value) -> Value {
    let Some(blob) = old.as_object_mut() else {
        return old;
    };
    let primary = blob.remove("ical_url_primary");
    let secondary = blob.remove("ical_url_secondary");
    let feeds_json = blob.remove("feeds_json");
    let rows = |value: Option<&Value>| -> Vec<CalendarRow> {
        value
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|row| serde_json::from_value(row.clone()).unwrap_or_default())
            .collect()
    };
    let mut calendars = rows(blob.get("calendars"));
    if feeds(&calendars).is_empty() {
        calendars = [primary, secondary]
            .iter()
            .enumerate()
            .filter_map(|(index, url)| {
                let url = trim_url(url.as_ref()?.as_str()?)?;
                Some(CalendarRow {
                    id: format!("cal-{}", index + 1),
                    url: url.to_string(),
                    ..CalendarRow::default()
                })
            })
            .collect();
    }
    if calendars.is_empty() {
        let listed = feeds_json
            .as_ref()
            .and_then(Value::as_str)
            .and_then(|raw| serde_json::from_str::<Value>(raw).ok());
        calendars = rows(listed.as_ref());
    }
    if !calendars.is_empty() {
        blob.insert("calendars".into(), json!(calendars));
    }
    old
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::MockContext;
    use serde_json::json;

    /// The feeds an old KV blob yields once mapped onto the declared keys.
    fn legacy_calendars(raw: Value) -> Vec<CalendarFeed> {
        let config: Config = serde_json::from_value(legacy(raw)).expect("mapped");
        feeds(&config.calendars)
    }

    #[test]
    fn migrates_primary_secondary_urls() {
        let calendars = legacy_calendars(json!({
            "ical_url_primary": " https://a.ics ",
            "ical_url_secondary": "https://b.ics",
        }));
        assert_eq!(calendars.len(), 2);
        assert_eq!(calendars[0].id, "cal-1");
        assert_eq!(calendars[0].url, "https://a.ics");
        assert_eq!(calendars[1].id, "cal-2");
        assert_eq!(calendars[1].url, "https://b.ics");
        assert_eq!(calendars[0].format, CalendarFormat::Generic);
    }

    #[test]
    fn migrates_feeds_json() {
        let calendars = legacy_calendars(json!({
            "feeds_json": r#"[{"url":"https://x.ics"},{"url":"https://y.ics"},{"url":"https://z.ics"}]"#,
        }));
        assert_eq!(calendars.len(), 3);
        assert_eq!(calendars[0].url, "https://x.ics");
    }

    #[test]
    fn calendars_list_wins_over_legacy() {
        let calendars = legacy_calendars(json!({
            "calendars": [{ "id": "c1", "url": "https://only.ics", "label": "Airbnb", "format": "airbnb" }],
            "ical_url_primary": "https://legacy.ics",
            "ical_url_secondary": "https://legacy2.ics",
            "last_sync_at": "2026-07-23T08:12:00Z",
        }));
        assert_eq!(calendars.len(), 1);
        assert_eq!(calendars[0].url, "https://only.ics");
        assert_eq!(calendars[0].format, CalendarFormat::Airbnb);
    }

    #[test]
    fn missing_format_detects_from_airbnb_url() {
        let calendars = feeds(&[CalendarRow {
            id: "c1".into(),
            url: "https://www.airbnb.com/calendar/ical/1.ics".into(),
            ..CalendarRow::default()
        }]);
        assert_eq!(calendars[0].format, CalendarFormat::Airbnb);
    }

    #[test]
    fn explicit_generic_is_kept_even_on_airbnb_url() {
        let calendars = feeds(&[CalendarRow {
            id: "c1".into(),
            url: "https://www.airbnb.com/calendar/ical/1.ics".into(),
            format: "generic".into(),
            ..CalendarRow::default()
        }]);
        assert_eq!(calendars[0].format, CalendarFormat::Generic);
    }

    /// What the host form sends: strings only, a removed row blank, no format.
    #[test]
    fn form_rows_are_normalized() {
        let config: Config = serde_json::from_value(json!({ "calendars": [
            { "id": "a", "channel": "", "label": "", "url": "  " },
            { "id": "", "channel": "booking", "label": " Booking ", "url": "https://example.com/a.ics" },
            { "id": "", "channel": "", "label": "", "url": "" },
        ]}))
        .expect("form rows");
        let config = ModuleConfig {
            calendars: feeds(&config.calendars),
        };
        assert_eq!(config.connected_calendars().len(), 1);
        assert!(config.has_any_feed());
        let feed = &config.calendars[0];
        assert_eq!(feed.id, "cal-2");
        assert_eq!(feed.label.as_deref(), Some("Booking"));
        assert_eq!(config.format_for_id("cal-2"), CalendarFormat::Booking);
        assert_eq!(
            config.channel_for_id("cal-2"),
            (BookingChannel::Booking, ChannelSignal::HostOverride)
        );
        assert_eq!(
            config.channel_for_id("missing"),
            (BookingChannel::Unknown, ChannelSignal::None)
        );
    }

    #[test]
    fn format_deduced_from_platform_when_url_is_neutral() {
        let neutral = "https://example.com/a.ics";
        assert_eq!(
            resolve_format("", neutral, BookingChannel::Airbnb),
            CalendarFormat::Airbnb
        );
        assert_eq!(
            resolve_format("", neutral, BookingChannel::Booking),
            CalendarFormat::Booking
        );
        // Direct / other sellers imply no dialect → generic.
        assert_eq!(
            resolve_format("", neutral, BookingChannel::Direct),
            CalendarFormat::Generic
        );
        // A stored explicit format still wins.
        assert_eq!(
            resolve_format("google", neutral, BookingChannel::Airbnb),
            CalendarFormat::Google
        );
    }

    #[test]
    fn feed_url_wins_over_the_platform() {
        assert_eq!(
            resolve_format(
                "",
                "https://calendar.google.com/calendar/ical/x/basic.ics",
                BookingChannel::Direct
            ),
            CalendarFormat::Google
        );
        assert_eq!(
            resolve_format(
                "",
                "https://www.airbnb.com/calendar/ical/1.ics",
                BookingChannel::Unknown
            ),
            CalendarFormat::Airbnb
        );
    }

    #[test]
    fn host_choice_is_a_host_override() {
        assert_eq!(
            resolve_channel("abritel-vrbo", "https://www.airbnb.com/calendar/ical/1.ics"),
            (BookingChannel::AbritelVrbo, ChannelSignal::HostOverride)
        );
        assert_eq!(
            resolve_channel("direct", "https://calendar.google.com/x/basic.ics"),
            (BookingChannel::Direct, ChannelSignal::HostOverride)
        );
    }

    #[test]
    fn undeclared_channel_falls_back_to_url_prefill() {
        assert_eq!(
            resolve_channel("", "https://www.airbnb.com/calendar/ical/1.ics"),
            (BookingChannel::Airbnb, ChannelSignal::FeedUrlHost)
        );
        assert_eq!(
            resolve_channel("unknown", "https://admin.booking.com/hotel/ical.html?t=x"),
            (BookingChannel::Booking, ChannelSignal::FeedUrlHost)
        );
    }

    #[test]
    fn transport_urls_never_prefill_a_platform() {
        for url in [
            "https://calendar.google.com/calendar/ical/x/basic.ics",
            "https://api.beds24.com/ical/x.ics",
            "https://example.com/a.ics",
        ] {
            assert_eq!(
                resolve_channel("", url),
                (BookingChannel::Unknown, ChannelSignal::None),
                "{url}"
            );
        }
    }

    #[test]
    fn google_format_migrates_to_an_unknown_channel() {
        let calendars = feeds(&[CalendarRow {
            id: "c1".into(),
            url: "https://calendar.google.com/calendar/ical/x/basic.ics".into(),
            ..CalendarRow::default()
        }]);
        assert_eq!(calendars[0].format, CalendarFormat::Google);
        assert_eq!(calendars[0].channel, BookingChannel::Unknown);
        assert_eq!(calendars[0].channel_signal, ChannelSignal::None);
    }

    #[test]
    fn stored_channel_survives_a_reload_with_its_provenance() {
        let calendars = legacy_calendars(json!({ "calendars": [{
            "id": "c1",
            "url": "https://www.airbnb.com/calendar/ical/1.ics",
            "format": "airbnb",
            "channel": "airbnb",
            "channel_signal": "feed-url-host",
        }]}));
        assert_eq!(calendars[0].channel, BookingChannel::Airbnb);
        assert_eq!(calendars[0].channel_signal, ChannelSignal::FeedUrlHost);
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

    #[test]
    fn only_the_secondary_url_keeps_its_id() {
        let calendars = legacy_calendars(json!({
            "ical_url_primary": " ",
            "ical_url_secondary": "https://b.ics",
        }));
        assert_eq!(calendars.len(), 1);
        assert_eq!(calendars[0].id, "cal-2");
    }

    #[test]
    fn blank_calendars_fall_back_to_the_older_keys() {
        let calendars = legacy_calendars(json!({
            "calendars": [{ "id": "c1", "url": " " }],
            "feeds_json": r#"[{"id":"f","url":"https://x.ics","label":"X"}]"#,
        }));
        assert_eq!(calendars.len(), 1);
        assert_eq!(calendars[0].id, "f");
        assert_eq!(calendars[0].label.as_deref(), Some("X"));
    }

    #[test]
    fn nothing_to_import_invents_no_list() {
        // No `calendars` key: the platform keeps warning until the host saves one.
        assert_eq!(legacy(json!({ "feeds_json": "[oops" })), json!({}));
        assert_eq!(
            legacy(json!({ "last_sync_at": "x" })),
            json!({ "last_sync_at": "x" })
        );
    }

    /// The KV is read (through `legacy`) only while the platform sends no config; `{}` is a
    /// real, empty config.
    #[test]
    #[serial_test::serial]
    fn the_kv_is_read_through_legacy_until_the_platform_holds_the_config() {
        let blob = json!({ "ical_url_primary": "https://example.com/a.ics" });
        let bytes = serde_json::to_vec(&blob).unwrap();
        MockContext::host()
            .with_kv("config", bytes.clone())
            .run(|ctx| {
                let config = ModuleConfig::load(&ctx).unwrap();
                assert_eq!(config.calendars.len(), 1);
                assert_eq!(config.calendars[0].url, "https://example.com/a.ics");
            });
        MockContext::host()
            .with_kv("config", bytes)
            .with_config(&json!({}))
            .run(|ctx| assert!(ModuleConfig::load(&ctx).unwrap().calendars.is_empty()));
    }
}
