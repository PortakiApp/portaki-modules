//! Module commands — guest submit, host resolve.

use portaki_sdk::files::FileRef;
use portaki_sdk::host::email::{
    self, EmailAudience, LocalizedEmailText, ModuleEmailCta, ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::host::events;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::category;
use crate::email_text;
use crate::storage;

/// Arguments for `submit`.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct SubmitArgs {
    pub category: String,
    pub summary: String,
    #[serde(default)]
    pub details: Option<String>,
    /// `ImageUpload` value: `portaki-file:<uuid>`, empty when no photo was attached.
    #[serde(default)]
    pub photo: Option<String>,
}

#[portaki_sdk::command(name = "submit", guest)]
pub fn submit(ctx: Context, args: SubmitArgs) -> Result<()> {
    let stay_id = require_stay_id(&ctx)?;
    let category = category::parse_category(&args.category)?;
    let summary = require_summary(&args.summary)?;
    let details = normalize_optional(args.details);
    let photo = parse_photo(args.photo)?;

    let _ = storage::create(
        stay_id,
        category.clone(),
        summary.clone(),
        details.clone(),
        photo,
    )?;

    // Guest text is quoted within a fixed length; the stored report keeps it whole.
    let quoted_summary = email_text::quote_guest_text(&summary);
    let quoted_details = details.as_deref().map(email_text::quote_guest_text);
    let truncated = quoted_summary.truncated
        || quoted_details
            .as_ref()
            .is_some_and(|quoted| quoted.truncated);

    let mut body = format!("Catégorie : {category}\n\n{}", quoted_summary.text);
    if let Some(extra) = &quoted_details {
        body.push_str("\n\n");
        body.push_str(&extra.text);
    }
    if photo.is_some() {
        body.push_str("\n\nUne photo est jointe — visible dans le tableau de bord.");
    }

    // The report is saved: a refused email is logged, it does not fail the guest's submit.
    let sent = email::send(&SendEmailArgs {
        email_id: format!("submitted-{stay_id}"),
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
        stay_id: Some(stay_id),
        property_id: Some(ctx.property_id),
        action_url: None,
    });
    if let Err(error) = sent {
        email_text::log_send_failure("issue_report_host_email_failed", &error);
    }
    Ok(())
}

/// Arguments for host `resolve`.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct ResolveArgs {
    pub report_id: Uuid,
}

#[portaki_sdk::command(name = "resolve")]
pub fn resolve(ctx: Context, args: ResolveArgs) -> Result<()> {
    if ctx.guest.is_some() {
        return Err(PortakiError::Host("host_only".to_string()));
    }
    let report = storage::resolve(args.report_id)?;
    // Activity log of the workspace (journal).
    events::emit(
        crate::ids::WORKSPACE_ACTIVITY_RECORD,
        &serde_json::json!({
            "eventId": "issue-report.resolved",
            "propertyId": ctx.property_id,
            "payload": { "reportId": report.id, "stayId": report.stay_id, "category": report.category },
            "display": { "chips": [{ "label": "Signalement", "value": report.summary }] },
        }),
    )
}

fn require_summary(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(PortakiError::Host("summary_required".to_string()));
    }
    Ok(trimmed.to_string())
}

/// Only a platform reference is kept: any other value (an external URL…) is refused, so a guest
/// cannot make the host dashboard load an arbitrary image.
fn parse_photo(raw: Option<String>) -> Result<Option<FileRef>> {
    match normalize_optional(raw) {
        None => Ok(None),
        Some(value) => FileRef::parse(&value)
            .map(Some)
            .ok_or_else(|| PortakiError::Host("invalid_photo".to_string())),
    }
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
