//! Module-owned transactional emails via `host::email::send`.

use chrono::Duration;
use portaki_sdk::host::email::{
    self, EmailAudience, EmailBlock, EmailFormField, ModuleEmailCta, ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::config::{Deadline, ModuleConfig};
use crate::email_i18n;
use crate::show_when::{is_editable_until_checkin, is_form_available, opened_at};
use crate::storage;

/// Stable delivery id — orchestrator dedups per stay + module + email_id.
pub const FORM_AVAILABLE_EMAIL_ID: &str = "form-available";

/// Scheduled / stay-created / property-publish / config-update catch-up — send guest
/// mail only when the form is available and not yet completed. Orchestrator
/// re-dispatches after draft publish and after host `updateConfig` (KV promote) so a
/// newly opened `show_when` does not leave stays without `form-available` (dedup
/// claim prevents re-send).
pub fn send_form_available(ctx: &Context) -> Result<()> {
    let Some((stay_id, config)) = open_and_unanswered(ctx)? else {
        return Ok(());
    };

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

/// The stay and config when the form is open, editable and still unanswered — `None` otherwise.
fn open_and_unanswered(ctx: &Context) -> Result<Option<(Uuid, ModuleConfig)>> {
    let stay_id = ctx
        .guest
        .as_ref()
        .map(|guest| guest.session_id)
        .or_else(|| ctx.stay.as_ref().map(|stay| stay.stay_id))
        .ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))?;

    if storage::find_by_stay(stay_id)?.is_some() {
        return Ok(None);
    }

    let config = ModuleConfig::load(ctx)?;
    let checkin_at = ctx.stay.as_ref().and_then(|stay| stay.checkin_at);
    let now = time::now()?;
    // Once check-in has passed the form is locked: inviting the guest to fill it is pointless.
    if !is_editable_until_checkin(now, checkin_at)
        || !is_form_available(config.show_when, now, checkin_at)
    {
        return Ok(None);
    }
    Ok(Some((stay_id, config)))
}

/// La relance de la limite `choice` (§2.3) : une commande et un e-mail par limite, la plateforme
/// les envoie tous les trois — seule celle que l'hôte a choisie part. Comme `form-available`, il
/// faut un formulaire ouvert et sans réponse, et ouvert depuis un jour : une relance au même
/// passage que l'ouverture doublerait l'e-mail d'ouverture. Et rien après la limite : un séjour
/// réservé la veille n'est pas relancé (§9 cas 3).
pub fn send_reminder(ctx: &Context, choice: Deadline, email_id: &str) -> Result<()> {
    let Some((stay_id, config)) = open_and_unanswered(ctx)? else {
        return Ok(());
    };
    let Some(checkin) = ctx.stay.as_ref().and_then(|stay| stay.checkin_at) else {
        return Ok(());
    };
    let now = time::now()?;
    // Ouvert depuis 24 h au moins : `form-available` a eu son passage un jour plus tôt, et les
    // deux e-mails ne partent jamais au même passage quotidien.
    let open_for_a_day = opened_at(config.show_when, Some(checkin))
        .is_none_or(|open| now >= open + Duration::hours(24));
    if !config.reminder
        || config.deadline != choice
        || !open_for_a_day
        || now >= choice.at(checkin, &ctx.property.timezone)
    {
        return Ok(());
    }

    email::send(&SendEmailArgs {
        email_id: email_id.into(),
        audience: EmailAudience::Guest,
        content: ModuleEmailSdui {
            subject: email_i18n::text("email.reminder.subject"),
            eyebrow: Some(email_i18n::text("email.formAvailable.eyebrow")),
            title: Some(email_i18n::text("email.reminder.title")),
            body: email_i18n::text(&format!("email.reminder.body.{}", choice.as_wire())),
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
