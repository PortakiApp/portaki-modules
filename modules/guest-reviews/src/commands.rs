//! Module commands — configuration and Portaki review submit.

use portaki_sdk::host;
use portaki_sdk::host::email::{
    self, EmailAudience, LocalizedEmailText, ModuleEmailCta, ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{load_config, normalize_url, save_config, Localized, ModuleConfig};

/// Arguments for `updateConfig` (flat form fields from host SDUI Save).
///
/// Platform / QR flags are `Option<bool>` so a missing key keeps the KV value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigArgs {
    #[serde(default)]
    pub platform_airbnb: Option<bool>,
    #[serde(default)]
    pub platform_portaki: Option<bool>,
    /// Legacy exclusive channel — migrated when platform toggles are absent.
    #[serde(default)]
    pub review_channel: String,
    #[serde(default)]
    pub show_qr_code: Option<bool>,
    #[serde(default)]
    pub airbnb_review_url: String,
    #[serde(default)]
    pub thank_you_message: String,
}

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    let lang = Localized::lang_code(&ctx.locale);
    let existing = load_config().unwrap_or_default();

    let (platform_airbnb, platform_portaki) = resolve_platform_args(&args, &existing);
    if !platform_airbnb && !platform_portaki {
        return Err(PortakiError::Host(
            "select_at_least_one_review_platform".into(),
        ));
    }

    // Airbnb section is omitted from the form when the toggle is off — keep the
    // stored URL so re-enabling Airbnb does not wipe a previous link.
    let airbnb_review_url = if platform_airbnb || !args.airbnb_review_url.trim().is_empty() {
        args.airbnb_review_url.trim().to_string()
    } else {
        existing.airbnb_review_url.clone()
    };

    if platform_airbnb && normalize_url(&airbnb_review_url).is_none() {
        return Err(PortakiError::Host("airbnb_review_url_required".into()));
    }

    let mut thank_you_message = existing.thank_you_message;
    thank_you_message.set(&lang, args.thank_you_message.trim().to_string());

    save_config(&ModuleConfig {
        platform_airbnb,
        platform_portaki,
        show_qr_code: args.show_qr_code.unwrap_or(existing.show_qr_code),
        airbnb_review_url,
        thank_you_message,
    })
}

fn resolve_platform_args(args: &UpdateConfigArgs, existing: &ModuleConfig) -> (bool, bool) {
    if args.platform_airbnb.is_some() || args.platform_portaki.is_some() {
        return (
            args.platform_airbnb.unwrap_or(existing.platform_airbnb),
            args.platform_portaki.unwrap_or(existing.platform_portaki),
        );
    }

    if !args.review_channel.trim().is_empty() {
        return match args.review_channel.trim().to_ascii_lowercase().as_str() {
            "portaki" => (false, true),
            "both" => (true, true),
            "airbnb" => (true, false),
            _ => (existing.platform_airbnb, existing.platform_portaki),
        };
    }

    (existing.platform_airbnb, existing.platform_portaki)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitReviewArgs {
    pub rating: u8,
    #[serde(default)]
    pub comment: String,
}

const REVIEWS_KEY: &str = "reviews";

#[portaki_sdk::command(name = "submitReview")]
pub fn submit_review(ctx: Context, args: SubmitReviewArgs) -> Result<()> {
    let config = load_config().unwrap_or_default();
    if !config.portaki_feasible() {
        return Err(PortakiError::Host(
            "portaki_review_platform_not_enabled".into(),
        ));
    }

    if !(1..=5).contains(&args.rating) {
        return Err(PortakiError::Host(format!(
            "rating must be 1-5, got {}",
            args.rating
        )));
    }

    let comment = args.comment.trim().to_string();
    let mut entries: Vec<SubmitReviewArgs> = host::kv::get(REVIEWS_KEY)?
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default();

    entries.push(SubmitReviewArgs {
        rating: args.rating,
        comment: comment.clone(),
    });

    let bytes = serde_json::to_vec(&entries)
        .map_err(|error| PortakiError::Storage(format!("reviews serialize: {error}")))?;
    host::kv::set(REVIEWS_KEY, &bytes, None)?;

    let guest_name = ctx
        .guest
        .as_ref()
        .and_then(|g| g.display_name.clone())
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "Voyageur".to_string());

    let stars = "★".repeat(args.rating as usize) + &"☆".repeat(5 - args.rating as usize);
    let mut body = format!("{guest_name} — {stars} ({}/5)", args.rating);
    if !comment.is_empty() {
        body.push_str("\n\n");
        body.push_str(&comment);
    }

    let stay_id = ctx.guest.as_ref().map(|g| g.session_id);

    email::send(&SendEmailArgs {
        email_id: "review-submitted".into(),
        audience: EmailAudience::Host,
        content: ModuleEmailSdui {
            subject: LocalizedEmailText::new(
                "Vous avez reçu un nouvel avis",
                "You received a new review",
            ),
            eyebrow: Some(LocalizedEmailText::both("Avis")),
            title: Some(LocalizedEmailText::new(
                "Nouvel avis voyageur",
                "New guest review",
            )),
            body: LocalizedEmailText::both(body),
            cta: Some(ModuleEmailCta {
                label: LocalizedEmailText::new("Voir le logement", "View property"),
                url: None,
                portaki_action: None,
            }),
        },
        stay_id,
        property_id: Some(ctx.property_id),
        action_url: None,
    })?;

    Ok(())
}
