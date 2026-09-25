//! Host configuration, held by the platform (`#[portaki_sdk::config]`).
//!
//! Platforms are multi-select (`platform_airbnb` / `platform_portaki`). Guest CTAs
//! only appear for platforms that are both selected and feasible (Airbnb needs a URL).

use portaki_sdk::contracts::i18n::I18nText;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// How the guest-facing Airbnb channel is decided.
///
/// `Manual` (default) keeps the historical behaviour: platforms are shown per the host toggles.
/// `Auto` derives Airbnb availability from the stay's actual booking platform
/// (`ctx.stay.booking_channel`), so the Airbnb CTA only appears for stays booked on Airbnb.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChannelMode {
    /// Follow the host platform toggles exactly (today's behaviour).
    #[default]
    Manual,
    /// Derive Airbnb availability from the stay's `booking_channel`.
    Auto,
}

/// Legacy exclusive channel — mapped onto the platform toggles by [`legacy`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LegacyReviewChannel {
    Airbnb,
    Portaki,
    Both,
}

impl LegacyReviewChannel {
    fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "portaki" => Self::Portaki,
            "both" => Self::Both,
            _ => Self::Airbnb,
        }
    }

    fn platforms(self) -> (bool, bool) {
        match self {
            Self::Airbnb => (true, false),
            Self::Portaki => (false, true),
            Self::Both => (true, true),
        }
    }
}

/// The keys are the names of the host form fields: the platform takes `updateConfig` itself.
/// The Airbnb URL is only recommended while Airbnb is selected: `publishReadiness` says so.
#[portaki_sdk::config(legacy = legacy)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    /// How the Airbnb channel availability is decided (`manual` = toggles, `auto` = derive from
    /// stay). No host form sets it: left out of the declared config.
    pub channel_mode: ChannelMode,
    /// Collect reviews via Airbnb link + optional QR.
    #[field(label = "host.channel.airbnb")]
    pub platform_airbnb: bool,
    /// Collect reviews via in-booklet Portaki star form. On by default so the post-stay surface
    /// always has an actionable channel (Airbnb needs a URL and is off until one is set).
    #[field(label = "host.channel.portaki")]
    pub platform_portaki: bool,
    #[field(label = "host.qr.label")]
    pub show_qr_code: bool,
    #[field(kind = "url", label = "host.airbnb.label")]
    pub airbnb_review_url: String,
    #[field(label = "host.thanks.label")]
    pub thank_you_message: I18nText,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            channel_mode: ChannelMode::Manual,
            platform_airbnb: true,
            platform_portaki: true,
            show_qr_code: true,
            airbnb_review_url: String::new(),
            thank_you_message: I18nText::default(),
        }
    }
}

/// The old KV blob, onto the declared keys: platforms chosen with the pre-toggle
/// `review_channel` (neither: Airbnb only, as it was), a URL without its scheme (not a valid
/// `url`), a message per language keyed by any locale spelling (`fr-FR`, `EN`).
fn legacy(old: Value) -> Value {
    let Value::Object(mut old) = old else {
        return old;
    };
    let flag = |old: &Map<String, Value>, key: &str| old.get(key).and_then(Value::as_bool);
    let (airbnb, portaki) = resolve_platforms(
        flag(&old, "platform_airbnb"),
        flag(&old, "platform_portaki"),
        old.remove("review_channel")
            .and_then(|channel| channel.as_str().map(str::to_string)),
    );
    old.insert("platform_airbnb".into(), airbnb.into());
    old.insert("platform_portaki".into(), portaki.into());
    if let Some(url) = old
        .get("airbnb_review_url")
        .and_then(Value::as_str)
        .and_then(normalize_url)
    {
        old.insert("airbnb_review_url".into(), url.into());
    }
    match old.remove("thank_you_message") {
        Some(Value::Object(texts)) => {
            let texts: Map<String, Value> = texts
                .into_iter()
                .filter_map(|(locale, text)| {
                    let language = locale.split(['-', '_']).next()?.trim().to_ascii_lowercase();
                    let text = text.as_str()?.trim();
                    (!language.is_empty() && !text.is_empty()).then(|| (language, text.into()))
                })
                .collect();
            old.insert("thank_you_message".into(), texts.into());
        }
        Some(text @ Value::String(_)) => {
            old.insert("thank_you_message".into(), text);
        }
        _ => {}
    }
    Value::Object(old)
}

impl ModuleConfig {
    /// True when no selected platform can actually run for the guest.
    pub fn is_empty(&self) -> bool {
        !self.has_feasible_platform()
    }

    pub fn airbnb_url(&self) -> Option<String> {
        normalize_url(&self.airbnb_review_url)
    }

    /// Airbnb selected and a usable review URL is present.
    pub fn airbnb_feasible(&self) -> bool {
        self.platform_airbnb && self.airbnb_url().is_some()
    }

    /// Portaki in-booklet form selected (always feasible once enabled).
    pub fn portaki_feasible(&self) -> bool {
        self.platform_portaki
    }

    pub fn has_feasible_platform(&self) -> bool {
        self.airbnb_feasible() || self.portaki_feasible()
    }

    /// Airbnb is selected but the review URL is missing — host must finish setup.
    pub fn airbnb_needs_url(&self) -> bool {
        self.platform_airbnb && self.airbnb_url().is_none()
    }

    /// Resolves which guest CTAs to show, honouring [`ChannelMode`].
    ///
    /// Returns `(show_airbnb, show_portaki)`.
    ///
    /// - `Manual`: exactly today's behaviour — each platform per its toggle/feasibility.
    /// - `Auto` with a resolved `booking_channel`:
    ///   - `airbnb` → Airbnb shown when a review URL is available (channel matched);
    ///   - any other known channel → Airbnb hidden (the stay was not booked on Airbnb);
    ///   - Portaki always follows its own toggle.
    /// - `Auto` with `None`/`unknown` (older backend, or channel not resolved): falls back to
    ///   Manual behaviour so there is no crash and no empty-state regression.
    pub fn resolve_guest_platforms(&self, booking_channel: Option<&str>) -> (bool, bool) {
        let portaki = self.portaki_feasible();
        match self.channel_mode {
            ChannelMode::Manual => (self.airbnb_feasible(), portaki),
            ChannelMode::Auto => match booking_channel {
                Some("airbnb") => (self.airbnb_url().is_some(), portaki),
                Some(other) if !other.is_empty() && other != "unknown" => (false, portaki),
                // None / "unknown" / empty → behave like today (defensive fallback).
                _ => (self.airbnb_feasible(), portaki),
            },
        }
    }
}

fn resolve_platforms(
    platform_airbnb: Option<bool>,
    platform_portaki: Option<bool>,
    review_channel: Option<String>,
) -> (bool, bool) {
    if platform_airbnb.is_some() || platform_portaki.is_some() {
        return (
            platform_airbnb.unwrap_or(false),
            platform_portaki.unwrap_or(false),
        );
    }

    if let Some(channel) = review_channel.as_deref() {
        return LegacyReviewChannel::parse(channel).platforms();
    }

    // Match historical default: Airbnb-only.
    (true, false)
}

pub fn normalize_url(raw: &str) -> Option<String> {
    let t = raw.trim();
    if t.is_empty() {
        return None;
    }
    let with_scheme = if t.starts_with("http://") || t.starts_with("https://") {
        t.to_string()
    } else {
        format!("https://{t}")
    };
    if with_scheme.starts_with("http://") || with_scheme.starts_with("https://") {
        Some(with_scheme)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::MockContext;
    use serde_json::json;

    fn from_legacy(old: Value) -> ModuleConfig {
        serde_json::from_value(legacy(old)).unwrap()
    }

    #[test]
    fn legacy_maps_the_channel_the_url_and_the_message() {
        let mapped = legacy(json!({
            "review_channel": "portaki",
            "airbnb_review_url": "airbnb.com/users/review/1",
            "thank_you_message": { "fr-FR": " Merci ", "EN": "Thanks", "de": "", "it": 3 },
            "show_qr_code": false
        }));
        assert_eq!(
            mapped,
            json!({
                "platform_airbnb": false,
                "platform_portaki": true,
                "airbnb_review_url": "https://airbnb.com/users/review/1",
                "thank_you_message": { "fr": "Merci", "en": "Thanks" },
                "show_qr_code": false
            })
        );
        let config: ModuleConfig = serde_json::from_value(mapped).unwrap();
        assert_eq!(config.thank_you_message.get("en-US"), "Thanks");
        assert!(!config.show_qr_code);
    }

    #[test]
    fn legacy_keeps_a_plain_message_and_drops_a_null_one() {
        let config = from_legacy(json!({ "thank_you_message": "Merci !" }));
        assert_eq!(config.thank_you_message.get("en"), "Merci !");
        assert!(from_legacy(json!({ "thank_you_message": null }))
            .thank_you_message
            .is_blank());
    }

    #[test]
    fn legacy_without_platforms_is_airbnb_only() {
        let config = from_legacy(json!({ "airbnb_review_url": "https://airbnb.com/r/1" }));
        assert!(config.platform_airbnb);
        assert!(!config.platform_portaki);
        assert!(config.show_qr_code);
        assert_eq!(config.airbnb_review_url, "https://airbnb.com/r/1");
        // Any unknown channel read as Airbnb, as before.
        let config = from_legacy(json!({ "review_channel": "email" }));
        assert!(config.platform_airbnb && !config.platform_portaki);
    }

    /// Until the platform holds the config, `load` reads the KV through `legacy`; once it does,
    /// even empty, the platform's value wins and the KV is not read.
    #[test]
    #[serial_test::serial]
    fn the_kv_is_read_through_legacy_until_the_platform_holds_the_config() {
        let old = json!({
            "review_channel": "portaki",
            "airbnb_review_url": "airbnb.com/users/review/1",
            "thank_you_message": { "fr": "Merci", "en": "Thanks" },
            "show_qr_code": false
        });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .run(|ctx| {
                let config = ModuleConfig::load(&ctx).unwrap();
                assert!(!config.platform_airbnb);
                assert!(config.platform_portaki);
                assert_eq!(
                    config.airbnb_review_url,
                    "https://airbnb.com/users/review/1"
                );
                assert_eq!(config.thank_you_message.get("en"), "Thanks");
            });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .with_config(&json!({}))
            .run(|ctx| {
                assert_eq!(ModuleConfig::load(&ctx).unwrap(), ModuleConfig::default());
            });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .with_config(&json!({
                "platform_airbnb": true,
                "platform_portaki": false,
                "airbnb_review_url": "",
                "thank_you_message": "Bye"
            }))
            .run(|ctx| {
                let config = ModuleConfig::load(&ctx).unwrap();
                assert!(config.platform_airbnb);
                assert!(!config.platform_portaki);
                assert_eq!(config.airbnb_review_url, "");
                assert_eq!(config.thank_you_message.get("en"), "Bye");
            });
    }

    #[test]
    fn migrates_legacy_both_channel() {
        let cfg = from_legacy(json!({
            "review_channel": "both",
            "airbnb_review_url": "https://airbnb.com/users/review/1"
        }));
        assert!(cfg.platform_airbnb);
        assert!(cfg.platform_portaki);
        assert!(cfg.airbnb_feasible());
    }

    #[test]
    fn platform_flags_win_over_legacy_channel() {
        let cfg = from_legacy(json!({
            "review_channel": "both",
            "platform_airbnb": false,
            "platform_portaki": true
        }));
        assert!(!cfg.platform_airbnb);
        assert!(cfg.platform_portaki);
        // One flag set: the other one is off, as before.
        let cfg = from_legacy(json!({ "platform_portaki": true }));
        assert!(!cfg.platform_airbnb);
    }

    #[test]
    fn channel_mode_defaults_to_manual_for_legacy_config() {
        let cfg = from_legacy(json!({
            "review_channel": "both",
            "airbnb_review_url": "https://airbnb.com/users/review/1"
        }));
        assert_eq!(cfg.channel_mode, ChannelMode::Manual);
        let cfg = from_legacy(json!({ "channel_mode": "auto" }));
        assert_eq!(cfg.channel_mode, ChannelMode::Auto);
    }

    #[test]
    fn manual_mode_ignores_booking_channel() {
        let cfg = ModuleConfig {
            channel_mode: ChannelMode::Manual,
            platform_airbnb: true,
            platform_portaki: true,
            airbnb_review_url: "https://airbnb.com/users/review/1".to_string(),
            ..ModuleConfig::default()
        };
        // Booking channel is irrelevant in Manual mode.
        assert_eq!(cfg.resolve_guest_platforms(Some("booking")), (true, true));
        assert_eq!(cfg.resolve_guest_platforms(None), (true, true));
    }

    #[test]
    fn auto_mode_shows_airbnb_only_for_airbnb_stays() {
        let cfg = ModuleConfig {
            channel_mode: ChannelMode::Auto,
            platform_airbnb: true,
            platform_portaki: true,
            airbnb_review_url: "https://airbnb.com/users/review/1".to_string(),
            ..ModuleConfig::default()
        };
        assert_eq!(cfg.resolve_guest_platforms(Some("airbnb")), (true, true));
        assert_eq!(cfg.resolve_guest_platforms(Some("booking")), (false, true));
        assert_eq!(cfg.resolve_guest_platforms(Some("direct")), (false, true));
    }

    #[test]
    fn auto_mode_airbnb_needs_url_even_for_airbnb_stays() {
        let cfg = ModuleConfig {
            channel_mode: ChannelMode::Auto,
            platform_airbnb: true,
            platform_portaki: false,
            airbnb_review_url: String::new(),
            ..ModuleConfig::default()
        };
        assert_eq!(cfg.resolve_guest_platforms(Some("airbnb")), (false, false));
    }

    #[test]
    fn auto_mode_falls_back_to_manual_when_channel_unknown() {
        let cfg = ModuleConfig {
            channel_mode: ChannelMode::Auto,
            platform_airbnb: true,
            platform_portaki: true,
            airbnb_review_url: "https://airbnb.com/users/review/1".to_string(),
            ..ModuleConfig::default()
        };
        // Older backend (None) or explicit unknown → today's behaviour, Airbnb still available.
        assert_eq!(cfg.resolve_guest_platforms(None), (true, true));
        assert_eq!(cfg.resolve_guest_platforms(Some("unknown")), (true, true));
    }

    #[test]
    fn airbnb_without_url_is_not_feasible() {
        let cfg = ModuleConfig {
            platform_airbnb: true,
            platform_portaki: false,
            airbnb_review_url: String::new(),
            ..ModuleConfig::default()
        };
        assert!(cfg.airbnb_needs_url());
        assert!(cfg.is_empty());
    }
}
