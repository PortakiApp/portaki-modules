//! `updateStatus` — la réponse de l'hôte.

use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::email::to_guest::reply_to_guest;
use crate::{email_text, status, storage};

/// Arguments for host `updateStatus`.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct UpdateStatusArgs {
    pub report_id: Uuid,
    pub status: String,
    /// La réponse au voyageur, du champ du formulaire : vide l'efface, absente la garde.
    #[serde(default, alias = "host_reply", skip_serializing_if = "Option::is_none")]
    pub host_reply: Option<String>,
}

/// « Répondre » : la réponse de l'hôte part au voyageur (`host-reply-<uuid>`).
#[portaki_sdk::email(
    id = "host-reply",
    audience = EmailAudience::Guest,
    requires_guest_email,
    description_key = "email.host-reply.description"
)]
#[portaki_sdk::command(
    name = "updateStatus",
    example(
        label = "Réassort fait",
        input = r#"{"reportId":"9d8c7b6a-5f4e-4d3c-8b2a-1f0e9d8c7b6a","status":"restocked"}"#
    ),
    example(
        label = "Prévu, avec une réponse",
        input = r#"{"reportId":"9d8c7b6a-5f4e-4d3c-8b2a-1f0e9d8c7b6a","status":"planned","hostReply":"Je passe ce soir vers 18 h."}"#
    )
)]
pub fn update_status(ctx: Context, args: UpdateStatusArgs) -> Result<()> {
    if ctx.guest.is_some() {
        return Err(PortakiError::Host("host_only".to_string()));
    }

    let status = status::parse_status(&args.status)?;
    let before = storage::find_report(args.report_id)?
        .and_then(|report| report.host_reply)
        .unwrap_or_default();
    let report = storage::update_status(args.report_id, status, args.host_reply)?;
    // « Répondre » : une réponse nouvelle part au voyageur par e-mail. Celle déjà envoyée ne
    // repart pas quand l'hôte passe ensuite la demande « Livré ».
    if let Some(reply) = report
        .host_reply
        .as_deref()
        .filter(|reply| *reply != before)
    {
        if let Err(error) = reply_to_guest(&report, reply) {
            email_text::log_send_failure("consumables_guest_reply_email_failed", &error);
        }
    }
    Ok(())
}
