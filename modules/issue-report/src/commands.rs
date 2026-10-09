//! Module commands — guest submit, host add and resolve.

use portaki_sdk::files::FileRef;
use portaki_sdk::host::email::{
    self, EmailAudience, EmailBlock, EmailPair, EmailTone, LocalizedEmailText, ModuleEmailCta,
    ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::host::events;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::category::Category;
use crate::email_text;
use crate::entities::IssueReport;
use crate::storage;

/// Arguments for `submit`.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct SubmitArgs {
    pub category: Category,
    pub summary: String,
    #[serde(default)]
    pub details: Option<String>,
    /// `ImageUpload` value: `portaki-file:<uuid>`, empty when no photo was attached.
    #[serde(default)]
    pub photo: Option<String>,
}

#[portaki_sdk::command(
    name = "submit",
    guest,
    example(
        label = "Four en panne",
        input = r#"{"category":"appliance","summary":"Le four ne chauffe plus","details":"Le voyant s'allume mais la température ne monte pas."}"#
    ),
    example(
        label = "Bruit la nuit",
        input = r#"{"category":"noise","summary":"Musique forte chez les voisins après minuit"}"#
    )
)]
pub fn submit(ctx: Context, args: SubmitArgs) -> Result<()> {
    let stay_id = require_stay_id(&ctx)?;
    let category = args.category.as_str();
    // Une pastille retirée par l'hôte ne passe pas : un formulaire ouvert dans un téléphone avant
    // qu'il la retire les proposait encore, et le refus tient à la configuration, pas à l'écran.
    let config = crate::config::ModuleConfig::load(&ctx)?;
    if !config.offers(category) {
        return Err(PortakiError::Host("category_not_offered".to_string()));
    }
    // Hors des périodes choisies, le formulaire n'est pas proposé : un écran resté ouvert ne
    // passe pas non plus.
    let stay = ctx.stay.as_ref();
    if !config.open_at(
        portaki_sdk::host::time::now()?,
        stay.and_then(|stay| stay.checkin_at),
        stay.and_then(|stay| stay.checkout_at),
    ) {
        return Err(PortakiError::Host("out_of_phase".to_string()));
    }
    let summary = require_summary(&args.summary)?;
    let details = normalize_optional(args.details);
    let photo = if config.photo_allowed() {
        parse_photo(args.photo)?
    } else {
        None
    };

    let _ = storage::create(
        stay_id,
        category.to_string(),
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

    let mut body = quoted_summary.text.clone();
    if let Some(extra) = &quoted_details {
        body.push_str("\n\n");
        body.push_str(&extra.text);
    }
    let mut blocks = vec![EmailBlock::rows([EmailPair::new(
        LocalizedEmailText::new("Catégorie", "Category"),
        LocalizedEmailText::both(category),
    )])];
    if photo.is_some() {
        blocks.push(EmailBlock::note(
            EmailTone::Info,
            LocalizedEmailText::new(
                "Une photo est jointe — visible dans le tableau de bord.",
                "A photo is attached — see it in the dashboard.",
            ),
        ));
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
            blocks,
            ..Default::default()
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

#[portaki_sdk::command(
    name = "resolve",
    example(
        label = "Signalement traité",
        input = r#"{"reportId":"9d8c7b6a-5f4e-4d3c-8b2a-1f0e9d8c7b6a"}"#
    )
)]
pub fn resolve(ctx: Context, args: ResolveArgs) -> Result<()> {
    resolve_report(&ctx, args.report_id).map(|_| ())
}

/// Clôt un signalement et l'inscrit au journal — `resolve` et la tâche « non traité ».
pub(crate) fn resolve_report(ctx: &Context, report_id: Uuid) -> Result<IssueReport> {
    require_host(ctx)?;
    let report = storage::resolve(report_id)?;
    // Activity log of the workspace (journal).
    events::emit(
        crate::ids::WORKSPACE_ACTIVITY_RECORD,
        &serde_json::json!({
            "eventId": "issue-report.resolved",
            "propertyId": ctx.property_id,
            "payload": { "reportId": report.id, "stayId": report.stay_id, "category": report.category },
            "display": { "chips": [{ "label": "Signalement", "value": report.summary }] },
        }),
    )?;
    Ok(report)
}

/// Arguments for host `add`.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct AddArgs {
    pub stay_id: Uuid,
    pub category: Category,
    pub summary: String,
    #[serde(default)]
    pub details: Option<String>,
}

/// Un signalement saisi par l'hôte depuis le séjour (constaté sur place, ou reçu par téléphone).
/// Ni e-mail ni filtre de catégorie ou de période : l'hôte sait ce qu'il note.
#[portaki_sdk::command(
    name = "add",
    example(
        label = "Signalé par téléphone",
        input = r#"{"stayId":"5d1a7e3c-2b9f-4c8d-a6e0-7f3b1c9d2e54","category":"appliance","summary":"Plus d'eau chaude"}"#
    )
)]
pub fn add(ctx: Context, args: AddArgs) -> Result<()> {
    require_host(&ctx)?;
    let summary = require_summary(&args.summary)?;
    let report = storage::create(
        args.stay_id,
        args.category.as_str().to_string(),
        summary,
        normalize_optional(args.details),
        None,
    )?;
    events::emit(
        crate::ids::WORKSPACE_ACTIVITY_RECORD,
        &serde_json::json!({
            "eventId": "issue-report.added",
            "propertyId": ctx.property_id,
            "payload": { "reportId": report.id, "stayId": report.stay_id, "category": report.category },
            "display": { "chips": [{ "label": "Signalement", "value": report.summary }] },
        }),
    )
}

fn require_host(ctx: &Context) -> Result<()> {
    if ctx.guest.is_some() {
        return Err(PortakiError::Host("host_only".to_string()));
    }
    Ok(())
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
