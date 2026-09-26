//! Module commands — guest submit, host submitFound / updateStatus.

use portaki_sdk::host::events;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::description;
use crate::email_send;
use crate::email_text;
use crate::kind;
use crate::status;
use crate::storage;

/// Arguments for guest `submit`.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct SubmitArgs {
    pub kind: String,
    pub item_description: String,
    #[serde(default)]
    pub contact_hint: Option<String>,
    #[serde(default)]
    pub details: Option<String>,
}

#[portaki_sdk::command(
    name = "submit",
    guest,
    example(
        label = "Chargeur oublié",
        input = r#"{"kind":"lost","itemDescription":"Chargeur de téléphone blanc","details":"Sans doute branché près du lit de la chambre 2."}"#
    ),
    example(
        label = "Objet trouvé sur place",
        input = r#"{"kind":"found","itemDescription":"Boucle d'oreille dorée","contactHint":"Posée sur la table de l'entrée"}"#
    )
)]
pub fn submit(ctx: Context, args: SubmitArgs) -> Result<()> {
    let stay_id = require_guest_stay_id(&ctx)?;
    let kind = kind::parse_kind(&args.kind)?;
    let item_description = require_description(&args.item_description)?;
    let contact_hint = normalize_optional(args.contact_hint);
    let details = normalize_optional(args.details);

    let _ = storage::create(
        stay_id,
        kind.clone(),
        item_description.clone(),
        contact_hint.clone(),
        details.clone(),
        status::DEFAULT.to_string(),
    )?;

    // The report is saved: a refused email is logged, it does not fail the guest's submit.
    if let Err(error) = email_send::notify_host_submitted(
        ctx.property_id,
        stay_id,
        &kind,
        &item_description,
        contact_hint.as_deref(),
        details.as_deref(),
    ) {
        email_text::log_send_failure("lost_found_host_email_failed", &error);
    }
    Ok(())
}

/// Arguments for host `submitFound` — one report per stay (shared description/status).
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct SubmitFoundArgs {
    /// Target stays (property-scoped). Empty when only [`Self::stay_id`] is set.
    #[serde(default)]
    pub stay_ids: Vec<Uuid>,
    /// Convenience single stay (merged into `stay_ids`).
    #[serde(default)]
    pub stay_id: Option<Uuid>,
    /// TipTap JSON or plain text — stored as-is; email gets plain extract.
    pub description: String,
    /// Wire status — default `to_collect` (« À récupérer »).
    #[serde(default)]
    pub status: Option<String>,
}

/// Each stay's guest hears of the found item (`host-found-<report id>`).
#[portaki_sdk::email(
    id = "host-found",
    audience = EmailAudience::Guest,
    requires_guest_email,
    description_key = "email.host-found.description"
)]
#[portaki_sdk::command(
    name = "submitFound",
    example(
        label = "Lunettes retrouvées",
        input = r#"{"stayId":"5d1a7e3c-2b9f-4c8d-a6e0-7f3b1c9d2e54","description":"Lunettes de soleil retrouvées sur la terrasse"}"#
    )
)]
pub fn submit_found(ctx: Context, args: SubmitFoundArgs) -> Result<()> {
    if ctx.guest.is_some() {
        return Err(PortakiError::Host("host_only".to_string()));
    }

    let stay_ids = resolve_stay_ids(&args)?;
    let description = require_description(&args.description)?;
    // Create always starts as « À récupérer » — status edits use `updateStatus`.
    let _ = args.status;
    let status = status::DEFAULT.to_string();
    let plain = description::to_plain_text(&description);
    if plain.is_empty() {
        return Err(PortakiError::Host("description_required".to_string()));
    }

    for stay_id in stay_ids {
        let report = storage::create(
            stay_id,
            "found".to_string(),
            description.clone(),
            None,
            None,
            status.clone(),
        )?;

        // The report is saved: a refused email is logged, the other stays still get theirs.
        if let Err(error) =
            email_send::notify_guest_host_found(ctx.property_id, stay_id, report.id, &plain)
        {
            email_text::log_send_failure("lost_found_guest_email_failed", &error);
        }
    }

    Ok(())
}

/// Scheduled J+2 tick — module owns gate + guest email content.
#[portaki_sdk::email(
    id = "checkout-j2",
    audience = EmailAudience::Guest,
    trigger = EmailTrigger::RelativeToCheckOut,
    offset_days = 2,
    requires_guest_email,
    description_key = "email.checkout-j2.description",
    skip_when = SkipWhen::GuestEmailMissing,
    skip_when = SkipWhen::StayCancelled
)]
#[portaki_sdk::command(
    name = "sendCheckoutFollowUp",
    example(label = "Relance deux jours après le départ")
)]
pub fn send_checkout_follow_up(ctx: Context, _args: EmptyArgs) -> Result<()> {
    email_send::send_checkout_follow_up(&ctx)
}

/// Arguments for host `updateStatus` — change workflow status after create.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct UpdateStatusArgs {
    pub report_id: Uuid,
    pub status: String,
}

#[portaki_sdk::command(name = "updateStatus")]
pub fn update_status(ctx: Context, args: UpdateStatusArgs) -> Result<()> {
    if ctx.guest.is_some() {
        return Err(PortakiError::Host("host_only".to_string()));
    }

    let status = status::parse_status(&args.status)?;
    let report = storage::update_status(args.report_id, status)?;
    // Activity log of the workspace (journal).
    events::emit(
        crate::ids::WORKSPACE_ACTIVITY_RECORD,
        &serde_json::json!({
            "eventId": "lost-found.status-updated",
            "propertyId": ctx.property_id,
            "payload": { "reportId": report.id, "stayId": report.stay_id, "status": report.status },
            "display": { "chips": [{
                "label": "Objet",
                "value": description::to_plain_text(&report.item_description),
            }] },
        }),
    )
}

fn resolve_stay_ids(args: &SubmitFoundArgs) -> Result<Vec<Uuid>> {
    let mut ids = args.stay_ids.clone();
    if let Some(stay_id) = args.stay_id {
        if !ids.contains(&stay_id) {
            ids.push(stay_id);
        }
    }
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
        return Err(PortakiError::Host("stay_ids_required".to_string()));
    }
    Ok(ids)
}

fn require_description(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || description::to_plain_text(trimmed).is_empty() {
        return Err(PortakiError::Host("description_required".to_string()));
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

fn require_guest_stay_id(ctx: &Context) -> Result<Uuid> {
    ctx.guest
        .as_ref()
        .map(|guest| guest.session_id)
        .ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))
}
