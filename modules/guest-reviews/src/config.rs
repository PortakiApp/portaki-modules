//! Host configuration stored in KV (`config` key).
//!
//! Platforms are multi-select (`platform_airbnb` / `platform_portaki`). Guest CTAs
//! only appear for platforms that are both selected and feasible (Airbnb needs a URL).

use portaki_sdk::host;
use portaki_sdk::Result;
use serde::{Deserialize, Serialize};

use crate::localized::deserialize_localized_field;

pub use crate::localized::Localized;

const CONFIG_KEY: &str = "config";

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

/// Legacy exclusive channel — still accepted on load, never written back.
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    /// How the Airbnb channel availability is decided (`manual` = toggles, `auto` = derive from stay).
    #[serde(default)]
    pub channel_mode: ChannelMode,
    /// Collect reviews via Airbnb link + optional QR.
    #[serde(default = "default_true")]
    pub platform_airbnb: bool,
    /// Collect reviews via in-booklet Portaki star form. On by default so the post-stay surface
    /// always has an actionable channel (Airbnb needs a URL and is off until one is set).
    #[serde(default = "default_true")]
    pub platform_portaki: bool,
    #[serde(default = "default_true")]
    pub show_qr_code: bool,
    #[serde(default)]
    pub airbnb_review_url: String,
    #[serde(default, deserialize_with = "deserialize_localized_field")]
    pub thank_you_message: Localized,
}

fn default_true() -> bool {
    true
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            channel_mode: ChannelMode::Manual,
            platform_airbnb: true,
            platform_portaki: true,
            show_qr_code: true,
            airbnb_review_url: String::new(),
            thank_you_message: Localized::default(),
        }
    }
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

/// Wire shape that accepts new platform toggles and legacy `review_channel`.
#[derive(Debug, Deserialize)]
struct RawModuleConfig {
    #[serde(default)]
    channel_mode: ChannelMode,
    #[serde(default)]
    platform_airbnb: Option<bool>,
    #[serde(default)]
    platform_portaki: Option<bool>,
    #[serde(default)]
    review_channel: Option<String>,
    #[serde(default = "default_true")]
    show_qr_code: bool,
    #[serde(default)]
    airbnb_review_url: String,
    #[serde(default, deserialize_with = "deserialize_localized_field")]
    thank_you_message: Localized,
}

impl From<RawModuleConfig> for ModuleConfig {
    fn from(raw: RawModuleConfig) -> Self {
        let (platform_airbnb, platform_portaki) = resolve_platforms(
            raw.platform_airbnb,
            raw.platform_portaki,
            raw.review_channel,
        );

        Self {
            channel_mode: raw.channel_mode,
            platform_airbnb,
            platform_portaki,
            show_qr_code: raw.show_qr_code,
            airbnb_review_url: raw.airbnb_review_url,
            thank_you_message: raw.thank_you_message,
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

pub fn load_config() -> Result<ModuleConfig> {
    let Some(bytes) = host::kv::get(CONFIG_KEY)? else {
        return Ok(ModuleConfig::default());
    };
    let raw: RawModuleConfig = serde_json::from_slice(&bytes).map_err(|error| {
        portaki_sdk::PortakiError::Storage(format!("invalid config JSON: {error}"))
    })?;
    Ok(ModuleConfig::from(raw))
}

pub fn save_config(config: &ModuleConfig) -> Result<()> {
    let bytes = serde_json::to_vec(config).map_err(|error| {
        portaki_sdk::PortakiError::Storage(format!("config serialize: {error}"))
    })?;
    host::kv::set(CONFIG_KEY, &bytes, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_legacy_both_channel() {
        let raw: RawModuleConfig = serde_json::from_value(serde_json::json!({
            "review_channel": "both",
            "airbnb_review_url": "https://airbnb.com/users/review/1"
        }))
        .unwrap();
        let cfg = ModuleConfig::from(raw);
        assert!(cfg.platform_airbnb);
        assert!(cfg.platform_portaki);
        assert!(cfg.airbnb_feasible());
    }

    #[test]
    fn platform_flags_win_over_legacy_channel() {
        let raw: RawModuleConfig = serde_json::from_value(serde_json::json!({
            "review_channel": "both",
            "platform_airbnb": false,
            "platform_portaki": true
        }))
        .unwrap();
        let cfg = ModuleConfig::from(raw);
        assert!(!cfg.platform_airbnb);
        assert!(cfg.platform_portaki);
    }

    #[test]
    fn channel_mode_defaults_to_manual_for_legacy_config() {
        let raw: RawModuleConfig = serde_json::from_value(serde_json::json!({
            "review_channel": "both",
            "airbnb_review_url": "https://airbnb.com/users/review/1"
        }))
        .unwrap();
        let cfg = ModuleConfig::from(raw);
        assert_eq!(cfg.channel_mode, ChannelMode::Manual);
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
