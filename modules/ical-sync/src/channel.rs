//! Booking channel detection — *who sold the stay*, not what shape the feed has.
//!
//! [`CalendarFormat`] answers the parsing question (which VEVENT rows are stays,
//! how a guest name is spelled). It is not a seller: `Google` is a mirror /
//! transport and `Generic` means unidentified. Only the three marketplace
//! formats map onto a [`BookingChannel`]; everything else falls through.
//!
//! The feed URL is deliberately absent from [`FeedChannelSignals`]. URL host is
//! a **configuration-time prefill** only — see [`prefill_from_url`] — because it
//! is wrong on channel-manager feeds (`beds24.com`, `smoobu.com` mask the
//! origin) and on a Google Calendar mirror. What the host saved from that
//! prefill reaches import as a declaration, carrying
//! [`ChannelSignal::FeedUrlHost`] so consumers can weigh it.

use portaki_sdk::contracts::booking_channel::{BookingChannel, ChannelSignal};

use crate::config::CalendarFormat;

/// Everything the module knows about one row's provenance at import time.
#[derive(Debug, Clone, Copy, Default)]
pub struct FeedChannelSignals<'a> {
    /// Feed shape declared by the host — a parsing dialect.
    pub declared_format: CalendarFormat,
    /// Selling platform declared (or prefilled and accepted) on the feed config.
    pub declared_channel: BookingChannel,
    /// Provenance of `declared_channel`: `HostOverride` | `FeedUrlHost` | `None`.
    pub declared_channel_signal: ChannelSignal,
    /// Calendar-scoped `PRODID`. Empty when the feed omits it.
    pub prodid: &'a str,
    /// Per-event `UID`. Empty when the VEVENT omits it.
    pub uid: &'a str,
}

/// Resolved channel plus the signal that decided it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DetectedChannel {
    pub code: BookingChannel,
    pub signal: ChannelSignal,
}

impl DetectedChannel {
    /// Nothing identified a seller.
    pub const UNKNOWN: DetectedChannel = DetectedChannel {
        code: BookingChannel::Unknown,
        signal: ChannelSignal::None,
    };
}

/// Resolves the booking channel of a single row. First known match wins.
///
/// 1. `UID` suffix — strongest, and it survives proxying: a feed re-exported by
///    a channel manager or mirrored into Google Calendar keeps the original UIDs.
/// 2. `PRODID` — strong when present, often absent. Calendar-scoped, so it
///    cannot discriminate a mixed feed.
/// 3. Host-declared platform (`HostOverride`) — an intention, but the only
///    signal that works on a channel-manager feed with opaque UIDs.
/// 4. Host-declared feed format, when that format names a marketplace.
/// 5. Platform prefilled from the feed URL at configuration time.
/// 6. Otherwise `unknown` / `none`.
///
/// `SUMMARY` / `DESCRIPTION` are never consulted — they are localised strings
/// that change without notice.
pub fn detect(signals: &FeedChannelSignals) -> DetectedChannel {
    if let Some(code) = channel_from_uid(signals.uid) {
        return DetectedChannel {
            code,
            signal: ChannelSignal::IcalUidSuffix,
        };
    }

    if let Some(code) = channel_from_prodid(signals.prodid) {
        return DetectedChannel {
            code,
            signal: ChannelSignal::IcalProdid,
        };
    }

    if signals.declared_channel.is_identified()
        && signals.declared_channel_signal == ChannelSignal::HostOverride
    {
        return DetectedChannel {
            code: signals.declared_channel,
            signal: ChannelSignal::HostOverride,
        };
    }

    if let Some(code) = channel_from_format(signals.declared_format) {
        return DetectedChannel {
            code,
            signal: ChannelSignal::FeedFormatDeclared,
        };
    }

    if signals.declared_channel.is_identified() {
        return DetectedChannel {
            code: signals.declared_channel,
            signal: signals.declared_channel_signal,
        };
    }

    DetectedChannel::UNKNOWN
}

/// Maps a feed shape onto a seller. `Google` / `Generic` describe transport, not
/// a marketplace, so they resolve to nothing.
pub fn channel_from_format(format: CalendarFormat) -> Option<BookingChannel> {
    match format {
        CalendarFormat::Airbnb => Some(BookingChannel::Airbnb),
        CalendarFormat::Booking => Some(BookingChannel::Booking),
        CalendarFormat::AbritelVrbo => Some(BookingChannel::AbritelVrbo),
        CalendarFormat::Google | CalendarFormat::Generic => None,
    }
}

/// Configuration-time prefill for the host platform selector.
///
/// Never called at import. Only marketplace hosts are recognised — a
/// `calendar.google.com` URL says nothing about who sold the stay.
pub fn prefill_from_url(url: &str) -> Option<BookingChannel> {
    let lower = url.to_ascii_lowercase();
    if lower.contains("airbnb.") {
        Some(BookingChannel::Airbnb)
    } else if lower.contains("booking.com") {
        Some(BookingChannel::Booking)
    } else if lower.contains("abritel.") || lower.contains("vrbo.") || lower.contains("homeaway.") {
        Some(BookingChannel::AbritelVrbo)
    } else {
        None
    }
}

/// Airbnb / Booking / Vrbo stamp their domain into the VEVENT `UID`.
fn channel_from_uid(uid: &str) -> Option<BookingChannel> {
    let lower = uid.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return None;
    }
    let suffix = lower.rsplit('@').next().unwrap_or(&lower);
    if suffix.contains("airbnb.") {
        Some(BookingChannel::Airbnb)
    } else if suffix.contains("booking.com") {
        Some(BookingChannel::Booking)
    } else if suffix.contains("abritel.")
        || suffix.contains("vrbo.")
        || suffix.contains("homeaway.")
        || suffix.contains("expediapartnercentral.")
    {
        Some(BookingChannel::AbritelVrbo)
    } else {
        None
    }
}

/// `PRODID:-//Airbnb Inc//Hosting Calendar 0.8.8//EN` and friends.
fn channel_from_prodid(prodid: &str) -> Option<BookingChannel> {
    let lower = prodid.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return None;
    }
    if lower.contains("airbnb") {
        Some(BookingChannel::Airbnb)
    } else if lower.contains("booking.com") {
        Some(BookingChannel::Booking)
    } else if lower.contains("vrbo") || lower.contains("homeaway") || lower.contains("abritel") {
        Some(BookingChannel::AbritelVrbo)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signals<'a>(uid: &'a str, prodid: &'a str) -> FeedChannelSignals<'a> {
        FeedChannelSignals {
            uid,
            prodid,
            ..Default::default()
        }
    }

    #[test]
    fn uid_suffix_wins_over_every_other_signal() {
        let detected = detect(&FeedChannelSignals {
            uid: "abc-123@airbnb.com",
            prodid: "-//Booking.com//Calendar//EN",
            declared_format: CalendarFormat::Booking,
            declared_channel: BookingChannel::Booking,
            declared_channel_signal: ChannelSignal::HostOverride,
        });
        assert_eq!(detected.code, BookingChannel::Airbnb);
        assert_eq!(detected.signal, ChannelSignal::IcalUidSuffix);
    }

    #[test]
    fn airbnb_uid_inside_a_feed_declared_generic_still_resolves_airbnb() {
        let detected = detect(&FeedChannelSignals {
            uid: "xyz@airbnb.com",
            declared_format: CalendarFormat::Generic,
            ..Default::default()
        });
        assert_eq!(detected.code, BookingChannel::Airbnb);
        assert_eq!(detected.signal, ChannelSignal::IcalUidSuffix);
    }

    #[test]
    fn prodid_decides_when_uid_is_opaque() {
        let detected = detect(&signals(
            "2f8c1d9e",
            "-//Airbnb Inc//Hosting Calendar 0.8.8//EN",
        ));
        assert_eq!(detected.code, BookingChannel::Airbnb);
        assert_eq!(detected.signal, ChannelSignal::IcalProdid);
    }

    #[test]
    fn host_override_beats_declared_format() {
        let detected = detect(&FeedChannelSignals {
            uid: "opaque-1",
            declared_format: CalendarFormat::Generic,
            declared_channel: BookingChannel::Direct,
            declared_channel_signal: ChannelSignal::HostOverride,
            ..Default::default()
        });
        assert_eq!(detected.code, BookingChannel::Direct);
        assert_eq!(detected.signal, ChannelSignal::HostOverride);
    }

    #[test]
    fn declared_format_beats_url_prefill() {
        let detected = detect(&FeedChannelSignals {
            uid: "opaque-2",
            declared_format: CalendarFormat::Booking,
            declared_channel: BookingChannel::Airbnb,
            declared_channel_signal: ChannelSignal::FeedUrlHost,
            ..Default::default()
        });
        assert_eq!(detected.code, BookingChannel::Booking);
        assert_eq!(detected.signal, ChannelSignal::FeedFormatDeclared);
    }

    #[test]
    fn url_prefill_is_last_resort_before_unknown() {
        let detected = detect(&FeedChannelSignals {
            uid: "opaque-3",
            declared_format: CalendarFormat::Generic,
            declared_channel: BookingChannel::AbritelVrbo,
            declared_channel_signal: ChannelSignal::FeedUrlHost,
            ..Default::default()
        });
        assert_eq!(detected.code, BookingChannel::AbritelVrbo);
        assert_eq!(detected.signal, ChannelSignal::FeedUrlHost);
    }

    #[test]
    fn google_mirror_resolves_unknown_not_google() {
        let detected = detect(&FeedChannelSignals {
            uid: "5k9v1p3q@google.com",
            prodid: "-//Google Inc//Google Calendar 70.9054//EN",
            declared_format: CalendarFormat::Google,
            ..Default::default()
        });
        assert_eq!(detected.code, BookingChannel::Unknown);
        assert_eq!(detected.signal, ChannelSignal::None);
    }

    #[test]
    fn no_signal_at_all_yields_unknown_none() {
        assert_eq!(detect(&signals("", "")), DetectedChannel::UNKNOWN);
        assert_eq!(detect(&signals("opaque-4", "")), DetectedChannel::UNKNOWN);
    }

    #[test]
    fn url_prefill_ignores_transport_hosts() {
        assert_eq!(
            prefill_from_url("https://www.airbnb.fr/calendar/ical/1.ics"),
            Some(BookingChannel::Airbnb)
        );
        assert_eq!(
            prefill_from_url("https://admin.booking.com/hotel/ical.html?t=x"),
            Some(BookingChannel::Booking)
        );
        assert_eq!(
            prefill_from_url("https://calendar.google.com/x/basic.ics"),
            None
        );
        assert_eq!(prefill_from_url("https://api.beds24.com/ical/x.ics"), None);
        assert_eq!(
            prefill_from_url("https://login.smoobu.com/ical/x.ics"),
            None
        );
    }

    #[test]
    fn feed_shape_only_maps_marketplaces() {
        assert_eq!(
            channel_from_format(CalendarFormat::AbritelVrbo),
            Some(BookingChannel::AbritelVrbo)
        );
        assert_eq!(channel_from_format(CalendarFormat::Google), None);
        assert_eq!(channel_from_format(CalendarFormat::Generic), None);
    }
}
