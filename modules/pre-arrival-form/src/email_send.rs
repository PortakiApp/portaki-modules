//! Module-owned transactional emails via `host::email::send`.

use portaki_sdk::host::email::{
    self, EmailAudience, EmailBlock, EmailFormField, ModuleEmailCta, ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;
use crate::email_i18n;
use crate::show_when::{is_editable_until_checkin, is_form_available};
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

    let config = ModuleConfig::load(ctx)?;
    let checkin_at = ctx.stay.as_ref().and_then(|stay| stay.checkin_at);
    let now = time::now()?;
    // Once check-in has passed the form is locked: inviting the guest to fill it is pointless.
    if !is_editable_until_checkin(now, checkin_at)
        || !is_form_available(config.show_when, now, checkin_at)
    {
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
            blocks: form_preview(&config),
            ..Default::default()
        },
        stay_id: Some(stay_id),
        property_id: None,
        action_url: None,
    })
}

/// The questions the host enabled, previewed as empty fields.
fn form_preview(config: &ModuleConfig) -> Vec<EmailBlock> {
    let asked = [
        (config.ask_arrival_time, "arrival"),
        (config.ask_guest_count, "guestCount"),
        (config.ask_occasion, "occasion"),
        (config.ask_allergies, "allergies"),
        (config.ask_special_needs, "specialNeeds"),
        (config.ask_id_document, "idDocument"),
    ];
    let fields: Vec<EmailFormField> = asked
        .into_iter()
        .filter(|(on, _)| *on)
        .map(|(_, key)| {
            EmailFormField::new(email_i18n::text(&format!(
                "email.formAvailable.field.{key}"
            )))
            .with_hint(email_i18n::text("email.formAvailable.fieldHint"))
        })
        .collect();
    if fields.is_empty() {
        return Vec::new();
    }
    vec![EmailBlock::fields(fields)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_lists_the_enabled_questions_in_every_locale() {
        let blocks = form_preview(&ModuleConfig::default());
        let [EmailBlock::Fields { items, .. }] = blocks.as_slice() else {
            panic!("{blocks:?}");
        };
        assert_eq!(items.len(), 4);
        for field in items {
            assert_eq!(field.label.translations.len(), 8, "{:?}", field.label);
            assert!(!field.label.fr.is_empty() && !field.label.en.is_empty());
        }

        let nothing = ModuleConfig {
            ask_arrival_time: false,
            ask_occasion: false,
            ask_allergies: false,
            ask_guest_count: false,
            ..ModuleConfig::default()
        };
        assert!(form_preview(&nothing).is_empty());
    }
}
