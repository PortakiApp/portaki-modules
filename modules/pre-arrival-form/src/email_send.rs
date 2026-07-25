//! Module-owned transactional emails via `host::email::send`.

use portaki_sdk::host::email::{
    self, EmailAudience, ModuleEmailCta, ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;

use crate::config::load_config;
use crate::email_i18n;
use crate::show_when::is_form_available;
use crate::storage;

/// Stable delivery id — orchestrator dedups per stay + module + email_id.
pub const FORM_AVAILABLE_EMAIL_ID: &str = "form-available";

/// Scheduled / stay-created / property-publish / config-update catch-up — send guest
/// mail only when the form is available and not yet completed. Orchestrator
/// re-dispatches after draft publish and after host `updateConfig` (KV promote) so a
/// newly opened `show_when` does not leave stays without `form-available` (dedup
/// claim prevents re-send).
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
            subject: email_i18n::text("email.formAvailable.subject"),
            eyebrow: Some(email_i18n::text("email.formAvailable.eyebrow")),
            title: Some(email_i18n::text("email.formAvailable.title")),
            body: email_i18n::text("email.formAvailable.body"),
            cta: Some(ModuleEmailCta {
                label: email_i18n::text("email.formAvailable.cta"),
                url: None,
                portaki_action: Some("open-module:pre-arrival-form:default".into()),
            }),
        },
        stay_id: Some(stay_id),
        property_id: None,
        action_url: None,
    })
}
