//! Module commands — submit issue report.

use portaki_sdk::host::email::{
    self, EmailAudience, LocalizedEmailText, ModuleEmailCta, ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::host::notify::{self, NotificationCategory, NotifyHostArgs};
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::category;
use crate::storage;

/// Arguments for `submit`.
#[portaki_sdk::wire]
pub struct SubmitArgs {
    pub category: String,
    pub summary: String,
    #[serde(default)]
    pub details: Option<String>,
}

#[portaki_sdk::command(name = "submit")]
pub fn submit(ctx: Context, args: SubmitArgs) -> Result<()> {
    let stay_id = require_stay_id(&ctx)?;
    let category = category::parse_category(&args.category)?;
    let summary = require_summary(&args.summary)?;
    let details = normalize_optional(args.details);

    let report = storage::create(stay_id, category.clone(), summary.clone(), details.clone())?;
    // Unique per report so each distinct report notifies the host (a constant
    // per-stay id would let only the first report ever reach the host).
    let notify_id = format!("submitted-{}", report.id);

    let mut body = format!("Catégorie : {category}\n\n{summary}");
    if let Some(extra) = &details {
        body.push_str("\n\n");
        body.push_str(extra);
    }

    email::send(&SendEmailArgs {
        email_id: notify_id.clone(),
        audience: EmailAudience::Host,
        content: ModuleEmailSdui {
            subject: LocalizedEmailText::new(
                "Un voyageur a signalé un problème",
                "A guest reported an issue",
            ),
            eyebrow: Some(LocalizedEmailText::both("Signalement")),
            title: Some(LocalizedEmailText::new(
                "Nouveau problème signalé",
                "New issue report",
            )),
            body: LocalizedEmailText::both(body.clone()),
            cta: Some(ModuleEmailCta {
                label: LocalizedEmailText::new("Voir le logement", "View property"),
                url: None,
                portaki_action: None,
            }),
        },
        stay_id: Some(stay_id),
        property_id: Some(ctx.property_id),
        action_url: None,
    })?;

    // Also raise a host inbox notification + push (guest-message category).
    // Best-effort: the report is already saved and emailed, so a notify failure
    // (e.g. capability ungranted or an older runtime) must not fail the submit.
    let _ = notify::notify_host(&NotifyHostArgs {
        notification_id: notify_id,
        title: LocalizedEmailText::new("Nouveau problème signalé", "New issue report"),
        body: LocalizedEmailText::both(body),
        category: NotificationCategory::GuestMessages,
        stay_id: Some(stay_id),
        property_id: Some(ctx.property_id),
        action_url: None,
    });
    Ok(())
}

fn require_summary(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(PortakiError::Host("summary_required".to_string()));
    }
    Ok(trimmed.to_string())
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn require_stay_id(ctx: &Context) -> Result<Uuid> {
    ctx.guest
        .as_ref()
        .map(|guest| guest.session_id)
        .ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))
}
