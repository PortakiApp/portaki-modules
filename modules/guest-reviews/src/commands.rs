//! Module commands — Portaki review submit.

use portaki_sdk::host;
use portaki_sdk::host::email::{
    self, EmailAudience, LocalizedEmailText, ModuleEmailCta, ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::ModuleConfig;
use crate::email_text;

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitReviewArgs {
    pub rating: u8,
    #[serde(default)]
    pub comment: String,
}

const REVIEWS_KEY: &str = "reviews";

/// One review as stored in KV. Reviews stored before the date was kept have no `at`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredReview {
    pub rating: u8,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub guest_name: Option<String>,
}

/// Every review of the property, oldest first.
pub fn load_reviews() -> Result<Vec<StoredReview>> {
    Ok(host::kv::get(REVIEWS_KEY)?
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default())
}

/// The guest has checked in — or the stay has no check-in date to say otherwise. Before
/// arrival there is no stay to review.
pub(crate) fn has_arrived(ctx: &Context) -> Result<bool> {
    match ctx.stay.as_ref().and_then(|stay| stay.checkin_at) {
        Some(checkin_at) => Ok(host::time::now()? >= checkin_at),
        None => Ok(true),
    }
}

#[portaki_sdk::command(name = "submitReview", guest)]
pub fn submit_review(ctx: Context, args: SubmitReviewArgs) -> Result<()> {
    let config = ModuleConfig::load(&ctx)?;
    if !config.portaki_feasible() {
        return Err(PortakiError::Host(
            "portaki_review_platform_not_enabled".into(),
        ));
    }
    if !has_arrived(&ctx)? {
        return Err(PortakiError::Host("review_before_arrival".into()));
    }

    if !(1..=5).contains(&args.rating) {
        return Err(PortakiError::Host(format!(
            "rating must be 1-5, got {}",
            args.rating
        )));
    }

    let comment = args.comment.trim().to_string();
    let guest_name = ctx
        .guest
        .as_ref()
        .and_then(|g| g.display_name.clone())
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty());
    let mut entries = load_reviews()?;
    entries.push(StoredReview {
        rating: args.rating,
        comment: comment.clone(),
        at: Some(host::time::now()?),
        guest_name: guest_name.clone(),
    });

    let bytes = serde_json::to_vec(&entries)
        .map_err(|error| PortakiError::Storage(format!("reviews serialize: {error}")))?;
    host::kv::set(REVIEWS_KEY, &bytes, None)?;

    let guest_name = guest_name.unwrap_or_else(|| "Voyageur".to_string());

    // Guest text is quoted within a fixed length; the stored review keeps it whole.
    let quoted_name = email_text::quote_guest_text(&guest_name);
    let quoted_comment = email_text::quote_guest_text(&comment);
    let truncated = quoted_name.truncated || quoted_comment.truncated;

    let stars = "★".repeat(args.rating as usize) + &"☆".repeat(5 - args.rating as usize);
    let mut body = format!("{} — {stars} ({}/5)", quoted_name.text, args.rating);
    if !comment.is_empty() {
        body.push_str("\n\n");
        body.push_str(&quoted_comment.text);
    }

    let stay_id = ctx.guest.as_ref().map(|g| g.session_id);

    // The review is saved: a refused email is logged, it does not fail the guest's submit.
    let sent = email::send(&SendEmailArgs {
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
                // No URL: with `property_id` set, the platform links the property page.
                label: email_text::cta_label(
                    truncated,
                    LocalizedEmailText::new("Voir le logement", "View property"),
                ),
                url: None,
                portaki_action: None,
            }),
        },
        stay_id,
        property_id: Some(ctx.property_id),
        action_url: None,
    });
    if let Err(error) = sent {
        email_text::log_send_failure("guest_reviews_host_email_failed", &error);
    }

    Ok(())
}
