//! Module-owned transactional emails via `host::email::send`.

use portaki_sdk::host::email::{
    self, EmailAudience, LocalizedEmailText, ModuleEmailCta, ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;

use crate::config::load_config;
use crate::show_when::is_form_available;
use crate::storage;

/// Stable delivery id — orchestrator dedups per stay + module + email_id.
pub const FORM_AVAILABLE_EMAIL_ID: &str = "form-available";

/// Scheduled / stay-created tick — send guest mail only when the form is available
/// and not yet completed.
pub fn send_form_available(ctx: &Context) -> Result<()> {
    let stay_id = ctx
        .guest
        .as_ref()
        .map(|guest| guest.session_id)
        .or_else(|| ctx.stay.as_ref().map(|stay| stay.stay_id))
        .ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))?;

    if storage::find_by_stay(stay_id)?.is_some() {
        return Ok(());
    }

    let config = load_config().unwrap_or_default();
    let checkin_at = ctx.stay.as_ref().and_then(|stay| stay.checkin_at);
    let now = time::now().unwrap_or_else(|_| chrono::Utc::now());
    if !is_form_available(config.show_when, now, checkin_at) {
        return Ok(());
    }

    email::send(&SendEmailArgs {
        email_id: FORM_AVAILABLE_EMAIL_ID.into(),
        audience: EmailAudience::Guest,
        content: ModuleEmailSdui {
            subject: LocalizedEmailText::new(
                "Un formulaire vous attend avant votre arrivée",
                "A form is ready before your arrival",
            ),
            eyebrow: Some(LocalizedEmailText::new(
                "Avant votre arrivée",
                "Before your arrival",
            )),
            title: Some(LocalizedEmailText::new(
                "Aidez-nous à préparer votre venue",
                "Help us prepare for your stay",
            )),
            body: LocalizedEmailText::new(
                "Votre hôte a préparé un court formulaire (horaire d’arrivée, allergies…).\n\nRépondez en 1 minute depuis votre livret séjour.",
                "Your host prepared a short form (arrival time, allergies…).\n\nAnswer in one minute from your stay booklet.",
            ),
            cta: Some(ModuleEmailCta {
                label: LocalizedEmailText::new("Ouvrir le formulaire", "Open the form"),
                url: None,
                portaki_action: Some("open-module:pre-arrival-form:default".into()),
            }),
        },
        stay_id: Some(stay_id),
        property_id: None,
        action_url: None,
    })
}
