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
    /// Quand la carte d'avis s'ouvre (spec §2.1) : à l'heure du départ, ou le lendemain à 10 h.
    #[field(kind = "select", options = ["checkout", "next_day_10h"], label = "host.askFrom.label")]
    pub ask_from: AskFrom,
    /// La page d'avis publique de l'hôte, sur n'importe quelle plateforme : Airbnb, Booking,
    /// Google, Abritel… Vide : la note seule.
    #[field(kind = "url", label = "host.reviewUrl.label")]
    pub review_url: String,
    /// Le voyageur peut écrire à l'hôte, sans le publier.
    #[field(label = "host.privateComment.label")]
    pub private_comment: bool,
    #[field(label = "host.thanks.label")]
    pub thank_you_message: I18nText,
    /// Carte « Page publique » : le bloc d'avis paraît sur la page publique du logement.
    ///
    /// À plat (`public_enabled`, pas `public.enabled`) : le tableau de bord n'imbrique que
    /// `liste.N.champ`, et la plateforme refuse une clé `public.enabled` (`config_field_unknown`).
    #[field(label = "host.public.enabled.label")]
    pub public_enabled: bool,
    /// Les avis montrés sur la page publique, 2 à 6 : le séjour de chacun (`stay:<id>:review`),
    /// pris parmi les avis consentis. Le choix multiple envoie un vrai tableau, que seul un champ
    /// `structured` accepte (un `text` exige une chaîne).
    #[field(kind = "structured", label = "host.public.reviews.label")]
    #[serde(deserialize_with = "id_list")]
    pub public_reviews: Vec<String>,

    /// How the booking-platform link is decided against the stay's channel (`auto`). No host
    /// form sets it: left out of the declared config.
    pub channel_mode: ChannelMode,
    /// Le lien Airbnb d'avant `review_url` : lu tant que `review_url` est vide.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub airbnb_review_url: String,
}

/// Les bornes du choix d'avis de la page publique.
pub const PUBLIC_REVIEWS_MIN: usize = 2;
pub const PUBLIC_REVIEWS_MAX: usize = 6;

/// Une liste d'identifiants, en tableau ou en texte : le choix multiple envoie un tableau ; les
/// brouillons d'avant le gardaient en JSON dans une chaîne (`"[\"a\",\"b\"]"`), ou en virgules.
fn id_list<'de, D>(deserializer: D) -> std::result::Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(match Value::deserialize(deserializer)? {
        Value::Array(items) => items
            .into_iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
        Value::String(text) => serde_json::from_str::<Vec<String>>(&text).unwrap_or_else(|_| {
            text.split(',')
                .map(|id| id.trim().to_string())
                .filter(|id| !id.is_empty())
                .collect()
        }),
        _ => Vec::new(),
    })
}

/// Quand demander l'avis.
#[portaki_sdk::params]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AskFrom {
    /// À l'heure du départ.
    #[default]
    Checkout,
    /// Le lendemain du départ, à 10 h (heure du logement).
    NextDay10h,
}

impl AskFrom {
    pub const fn as_wire(self) -> &'static str {
        match self {
            Self::Checkout => "checkout",
            Self::NextDay10h => "next_day_10h",
        }
    }
}

/// La plateforme d'un lien d'avis, lue sur son domaine (spec §4) — son nom sur le bouton.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewPlatform {
    Airbnb,
    Booking,
    Google,
    Abritel,
    Tripadvisor,
    /// Un autre site : le bouton dit seulement « Laisser un avis ».
    Other,
}

impl ReviewPlatform {
    pub fn of(url: &str) -> Self {
        let lower = url.trim().to_ascii_lowercase();
        let host = lower
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split(['/', '?', '#'])
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let is = |domain: &str| host == domain || host.ends_with(&format!(".{domain}"));
        if host.split('.').any(|part| part == "airbnb") {
            Self::Airbnb
        } else if is("booking.com") {
            Self::Booking
        } else if is("g.page") || is("goo.gl") || (host.split('.').any(|part| part == "google")) {
            Self::Google
        } else if is("abritel.fr") || is("vrbo.com") || is("homeaway.com") {
            Self::Abritel
        } else if host.split('.').any(|part| part == "tripadvisor") {
            Self::Tripadvisor
        } else {
            Self::Other
        }
    }

    /// Le canal de réservation que cette plateforme désigne (`booking_channel` du séjour).
    fn booking_channel(self) -> Option<&'static str> {
        match self {
            Self::Airbnb => Some("airbnb"),
            Self::Booking => Some("booking"),
            Self::Abritel => Some("vrbo"),
            Self::Google | Self::Tripadvisor | Self::Other => None,
        }
    }

    /// La clé du libellé du bouton.
    pub fn cta_key(self) -> &'static str {
        match self {
            Self::Airbnb => "i18n:guest.cta.airbnb",
            Self::Booking => "i18n:guest.cta.booking",
            Self::Google => "i18n:guest.cta.google",
            Self::Abritel => "i18n:guest.cta.abritel",
            Self::Tripadvisor => "i18n:guest.cta.tripadvisor",
            Self::Other => "i18n:guest.cta.other",
        }
    }
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            ask_from: AskFrom::Checkout,
            review_url: String::new(),
            private_comment: true,
            thank_you_message: I18nText::default(),
            channel_mode: ChannelMode::Manual,
            airbnb_review_url: String::new(),
            public_enabled: false,
            public_reviews: Vec::new(),
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
    // Les deux interrupteurs disparaissent : la note se propose toujours, le lien dès qu'il
    // existe. Le lien Airbnb ne devient le lien public que si Airbnb était choisi.
    let _ = portaki;
    old.remove("platform_airbnb");
    old.remove("platform_portaki");
    old.remove("show_qr_code");
    if let Some(url) = old
        .remove("airbnb_review_url")
        .as_ref()
        .and_then(Value::as_str)
        .and_then(normalize_url)
        .filter(|_| airbnb)
    {
        old.insert("review_url".into(), url.into());
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
    /// Le lien public, normalisé : `review_url`, sinon l'ancien lien Airbnb.
    pub fn public_url(&self) -> Option<String> {
        normalize_url(&self.review_url).or_else(|| normalize_url(&self.airbnb_review_url))
    }

    /// La plateforme du lien public.
    pub fn platform(&self) -> Option<ReviewPlatform> {
        self.public_url().map(|url| ReviewPlatform::of(&url))
    }

    /// Le lien se propose-t-il pour ce séjour ? À tous les voyageurs, quelle que soit leur note
    /// (spec §3 : filtrer selon la note est interdit). En mode `auto` seulement, le lien d'une
    /// plateforme de réservation ne se propose qu'aux séjours réservés sur elle : un voyageur
    /// venu par Booking ne peut pas laisser d'avis sur Airbnb.
    pub fn shows_link(&self, booking_channel: Option<&str>) -> bool {
        let Some(platform) = self.platform() else {
            return false;
        };
        match (
            self.channel_mode,
            platform.booking_channel(),
            booking_channel,
        ) {
            (ChannelMode::Auto, Some(wanted), Some(channel))
                if !channel.is_empty() && channel != "unknown" =>
            {
                channel == wanted
            }
            _ => true,
        }
    }

    /// La carte s'ouvre-t-elle déjà ? À l'heure du départ (le livret s'en charge déjà), ou le
    /// lendemain à 10 h, heure du logement. Sans départ connu, tout de suite.
    pub fn asks_now(
        &self,
        now: chrono::DateTime<chrono::Utc>,
        checkout: Option<chrono::DateTime<chrono::Utc>>,
        timezone: &str,
    ) -> bool {
        let Some(checkout) = checkout else {
            return true;
        };
        match self.ask_from {
            // Le livret n'ouvre l'écran de fin de séjour qu'après le départ : rien à ajouter.
            AskFrom::Checkout => true,
            AskFrom::NextDay10h => {
                let tz = portaki_sdk::host::time::PropertyTz::parse(timezone)
                    .or_else(|| portaki_sdk::host::time::PropertyTz::parse("Europe/Paris"));
                let Some(tz) = tz else {
                    return now >= checkout;
                };
                let next_day = tz.to_local(checkout).date_naive() + chrono::Days::new(1);
                let ten = next_day.and_hms_opt(10, 0, 0).expect("10:00 exists");
                now >= tz.from_local(ten)
            }
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
    let lower = t.to_ascii_lowercase();
    let with_scheme = if lower.starts_with("http://") || lower.starts_with("https://") {
        t.to_string()
    } else {
        format!("https://{t}")
    };
    if with_scheme.to_ascii_lowercase().starts_with("http") {
        Some(with_scheme)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use portaki_test_utils::MockContext;
    use serde_json::json;

    fn from_legacy(old: Value) -> ModuleConfig {
        serde_json::from_value(legacy(old)).unwrap()
    }

    fn with_link(url: &str, channel_mode: ChannelMode) -> ModuleConfig {
        ModuleConfig {
            review_url: url.into(),
            channel_mode,
            ..ModuleConfig::default()
        }
    }

    fn at(raw: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(raw)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn legacy_maps_the_airbnb_url_and_the_message_and_drops_the_switches() {
        let mapped = legacy(json!({
            "review_channel": "both",
            "airbnb_review_url": "airbnb.com/users/review/1",
            "thank_you_message": { "fr-FR": " Merci ", "EN": "Thanks", "de": "", "it": 3 },
            "show_qr_code": false
        }));
        assert_eq!(
            mapped,
            json!({
                "review_url": "https://airbnb.com/users/review/1",
                "thank_you_message": { "fr": "Merci", "en": "Thanks" }
            })
        );
        let config: ModuleConfig = serde_json::from_value(mapped).unwrap();
        assert_eq!(config.thank_you_message.get("en-US"), "Thanks");
        assert!(config.private_comment);
        assert_eq!(config.ask_from, AskFrom::Checkout);
    }

    #[test]
    fn legacy_portaki_only_keeps_no_public_link() {
        let config = from_legacy(json!({
            "review_channel": "portaki",
            "airbnb_review_url": "https://airbnb.com/users/review/1"
        }));
        assert_eq!(config.review_url, "");
        assert_eq!(config.airbnb_review_url, "");
        assert_eq!(config.public_url(), None);
        // Platform flags too: Airbnb off, the URL goes.
        let config = from_legacy(json!({
            "platform_airbnb": false,
            "platform_portaki": true,
            "airbnb_review_url": "https://airbnb.com/users/review/1"
        }));
        assert_eq!(config.public_url(), None);
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
        assert_eq!(config.review_url, "https://airbnb.com/r/1");
        assert_eq!(config.platform(), Some(ReviewPlatform::Airbnb));
        // Any unknown channel read as Airbnb, as before.
        let config = from_legacy(json!({
            "review_channel": "email",
            "airbnb_review_url": "airbnb.com/r/2"
        }));
        assert_eq!(config.review_url, "https://airbnb.com/r/2");
    }

    #[test]
    fn platform_flags_win_over_legacy_channel() {
        let config = from_legacy(json!({
            "review_channel": "both",
            "platform_airbnb": false,
            "platform_portaki": true,
            "airbnb_review_url": "https://airbnb.com/r/1"
        }));
        assert_eq!(config.public_url(), None);
        // Airbnb flag on beats a Portaki-only channel.
        let config = from_legacy(json!({
            "review_channel": "portaki",
            "platform_airbnb": true,
            "airbnb_review_url": "https://airbnb.com/r/1"
        }));
        assert_eq!(config.review_url, "https://airbnb.com/r/1");
    }

    /// Until the platform holds the config, `load` reads the KV through `legacy`; once it does,
    /// even empty, the platform's value wins and the KV is not read.
    #[test]
    #[serial_test::serial]
    fn the_kv_is_read_through_legacy_until_the_platform_holds_the_config() {
        let old = json!({
            "review_channel": "both",
            "airbnb_review_url": "airbnb.com/users/review/1",
            "thank_you_message": { "fr": "Merci", "en": "Thanks" },
            "show_qr_code": false
        });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .run(|ctx| {
                let config = ModuleConfig::load(&ctx).unwrap();
                assert_eq!(config.review_url, "https://airbnb.com/users/review/1");
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
                "review_url": "https://g.page/r/abc/review",
                "private_comment": false,
                "thank_you_message": "Bye"
            }))
            .run(|ctx| {
                let config = ModuleConfig::load(&ctx).unwrap();
                assert_eq!(config.platform(), Some(ReviewPlatform::Google));
                assert!(!config.private_comment);
                assert_eq!(config.thank_you_message.get("en"), "Bye");
            });
    }

    #[test]
    fn channel_mode_defaults_to_manual() {
        assert_eq!(ModuleConfig::default().channel_mode, ChannelMode::Manual);
        let config = from_legacy(json!({ "review_channel": "both" }));
        assert_eq!(config.channel_mode, ChannelMode::Manual);
        let config = from_legacy(json!({ "channel_mode": "auto" }));
        assert_eq!(config.channel_mode, ChannelMode::Auto);
    }

    #[test]
    fn the_public_url_is_review_url_else_the_old_airbnb_link() {
        let config = ModuleConfig {
            airbnb_review_url: "airbnb.fr/users/review/1".into(),
            ..ModuleConfig::default()
        };
        assert_eq!(
            config.public_url().as_deref(),
            Some("https://airbnb.fr/users/review/1")
        );
        let config = ModuleConfig {
            review_url: " www.booking.com/reviews/x ".into(),
            ..config
        };
        assert_eq!(
            config.public_url().as_deref(),
            Some("https://www.booking.com/reviews/x")
        );
        assert_eq!(ModuleConfig::default().public_url(), None);
        assert_eq!(ModuleConfig::default().platform(), None);
    }

    #[test]
    fn the_platform_is_read_on_the_domain() {
        for (url, platform) in [
            (
                "https://www.airbnb.fr/users/review/1",
                ReviewPlatform::Airbnb,
            ),
            ("https://airbnb.com/r/1", ReviewPlatform::Airbnb),
            (
                "https://www.booking.com/hotel/fr/x.html",
                ReviewPlatform::Booking,
            ),
            ("https://booking.com/reviews", ReviewPlatform::Booking),
            ("https://WWW.Booking.com/x", ReviewPlatform::Booking),
            ("https://g.page/r/abc/review", ReviewPlatform::Google),
            ("https://goo.gl/maps/xyz", ReviewPlatform::Google),
            (
                "https://www.google.com/maps/place/x",
                ReviewPlatform::Google,
            ),
            ("https://www.abritel.fr/location/1", ReviewPlatform::Abritel),
            ("https://www.vrbo.com/1234", ReviewPlatform::Abritel),
            ("https://www.homeaway.com/1234", ReviewPlatform::Abritel),
            (
                "https://www.tripadvisor.fr/Hotel_Review",
                ReviewPlatform::Tripadvisor,
            ),
            ("https://example.com/avis", ReviewPlatform::Other),
            // A domain that only contains a platform name is not that platform.
            ("https://notbooking.com/x", ReviewPlatform::Other),
            (
                "https://example.com/?ref=booking.com",
                ReviewPlatform::Other,
            ),
        ] {
            assert_eq!(ReviewPlatform::of(url), platform, "{url}");
        }
        assert_eq!(ReviewPlatform::Booking.cta_key(), "i18n:guest.cta.booking");
        assert_eq!(ReviewPlatform::Other.cta_key(), "i18n:guest.cta.other");
    }

    #[test]
    fn manual_mode_shows_the_link_whatever_the_channel() {
        let config = with_link("https://www.airbnb.fr/users/review/1", ChannelMode::Manual);
        for channel in [Some("airbnb"), Some("booking"), Some("direct"), None] {
            assert!(config.shows_link(channel), "{channel:?}");
        }
        assert!(!ModuleConfig::default().shows_link(Some("airbnb")));
    }

    #[test]
    fn auto_mode_shows_a_booking_platform_link_only_to_its_stays() {
        let airbnb = with_link("https://www.airbnb.fr/users/review/1", ChannelMode::Auto);
        assert!(airbnb.shows_link(Some("airbnb")));
        assert!(!airbnb.shows_link(Some("booking")));
        assert!(!airbnb.shows_link(Some("direct")));
        // Unknown channel: shown, as before.
        assert!(airbnb.shows_link(None));
        assert!(airbnb.shows_link(Some("")));
        assert!(airbnb.shows_link(Some("unknown")));

        let booking = with_link("https://www.booking.com/x", ChannelMode::Auto);
        assert!(booking.shows_link(Some("booking")));
        assert!(!booking.shows_link(Some("airbnb")));
        let vrbo = with_link("https://www.abritel.fr/x", ChannelMode::Auto);
        assert!(vrbo.shows_link(Some("vrbo")));
        assert!(!vrbo.shows_link(Some("airbnb")));

        // Google and others are not booking platforms: every stay gets them.
        let google = with_link("https://g.page/r/abc/review", ChannelMode::Auto);
        assert!(google.shows_link(Some("booking")));
        assert!(google.shows_link(Some("airbnb")));
        // No link, no button.
        assert!(!ModuleConfig {
            channel_mode: ChannelMode::Auto,
            ..ModuleConfig::default()
        }
        .shows_link(Some("airbnb")));
    }

    #[test]
    fn checkout_mode_leaves_the_timing_to_the_booklet() {
        // Le livret n'ouvre l'écran de fin de séjour qu'après le départ : le module n'ajoute rien.
        let config = ModuleConfig::default();
        let checkout = at("2026-08-29T08:00:00Z");
        assert!(config.asks_now(at("2026-08-29T07:59:59Z"), Some(checkout), "Europe/Paris"));
        assert!(config.asks_now(checkout, Some(checkout), "Europe/Paris"));
        // No checkout known: right away.
        assert!(config.asks_now(at("2020-01-01T00:00:00Z"), None, "Europe/Paris"));
    }

    #[test]
    fn next_day_mode_asks_the_day_after_at_ten_in_the_property_timezone() {
        let config = ModuleConfig {
            ask_from: AskFrom::NextDay10h,
            ..ModuleConfig::default()
        };
        // 10:00 Paris on the 29th → 10:00 Paris (08:00Z) on the 30th.
        let checkout = at("2026-08-29T08:00:00Z");
        assert!(!config.asks_now(at("2026-08-29T20:00:00Z"), Some(checkout), "Europe/Paris"));
        assert!(!config.asks_now(at("2026-08-30T07:59:59Z"), Some(checkout), "Europe/Paris"));
        assert!(config.asks_now(at("2026-08-30T08:00:00Z"), Some(checkout), "Europe/Paris"));
        // Unknown timezone: Paris.
        assert!(!config.asks_now(at("2026-08-30T07:59:59Z"), Some(checkout), "Mars/Olympus"));
        assert!(config.asks_now(at("2026-08-30T08:00:00Z"), Some(checkout), ""));

        // The local date counts: 02:00Z on the 29th is still the 28th in New York, so the
        // threshold is 10:00 New York on the 29th (14:00Z).
        let checkout = at("2026-08-29T02:00:00Z");
        let ny = "America/New_York";
        assert!(!config.asks_now(at("2026-08-29T13:59:59Z"), Some(checkout), ny));
        assert!(config.asks_now(at("2026-08-29T14:00:00Z"), Some(checkout), ny));
        // No checkout known: right away.
        assert!(config.asks_now(at("2020-01-01T00:00:00Z"), None, ny));
    }
}
