//! Module-owned transactional emails via `host::email::send`.
//!
//! Guest-typed text is quoted within a fixed length — the report keeps it whole; see
//! [`crate::email_text`].

use portaki_sdk::host::email::{
    self, EmailAudience, LocalizedEmailText, ModuleEmailCta, ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::description;
use crate::email_i18n;
use crate::email_text::{self, Quoted};
use crate::entities::LostFoundReport;
use crate::storage;

/// Longest run of joined declarations quoted in the J+2 follow-up, in chars.
///
/// Each declaration is already quoted within [`email_text::GUEST_TEXT_EMAIL_MAX_CHARS`]; a
/// stay with many of them would still overflow `limits::EMAIL_BODY_MAX_CHARS` without this.
const JOINED_DESCRIPTIONS_MAX_CHARS: usize = 3000;

/// Guest self-report → notify workspace owner (host audience — FR/EN).
pub fn notify_host_submitted(
    property_id: Uuid,
    stay_id: Uuid,
    kind: &str,
    item_description: &str,
    contact_hint: Option<&str>,
    details: Option<&str>,
) -> Result<()> {
    let description = email_text::quote_guest_text(item_description);
    let hint = contact_hint.map(email_text::quote_guest_text);
    let extra = details.map(email_text::quote_guest_text);
    let truncated = description.truncated
        || [&hint, &extra]
            .into_iter()
            .flatten()
            .any(|quoted| quoted.truncated);

    let mut body = format!(
        "Un voyageur a signalé un objet ({kind}) :\n\n{}",
        description.text
    );
    if let Some(hint) = &hint {
        body.push_str("\n\nContact / lieu : ");
        body.push_str(&hint.text);
    }
    if let Some(extra) = &extra {
        body.push_str("\n\nDétails : ");
        body.push_str(&extra.text);
    }

    email::send(&SendEmailArgs {
        email_id: format!("submitted-{stay_id}"),
        audience: EmailAudience::Host,
        content: ModuleEmailSdui {
            subject: LocalizedEmailText::new(
                "Un voyageur a signalé un objet perdu/trouvé",
                "A guest reported a lost or found item",
            ),
            eyebrow: Some(LocalizedEmailText::both("Objets perdus / trouvés")),
            title: Some(LocalizedEmailText::new("Nouveau signalement", "New report")),
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
        property_id: Some(property_id),
        action_url: None,
    })
}

/// Host-declared found item → notify guest (multi-locale).
pub fn notify_guest_host_found(
    property_id: Uuid,
    stay_id: Uuid,
    report_id: Uuid,
    plain_description: &str,
) -> Result<()> {
    let description = email_text::quote_guest_text(plain_description);
    let vars = [("description", description.text.as_str())];
    email::send(&SendEmailArgs {
        email_id: format!("host-found-{report_id}"),
        audience: EmailAudience::Guest,
        content: ModuleEmailSdui {
            subject: email_i18n::text("email.hostFound.subject"),
            eyebrow: Some(email_i18n::text("email.hostFound.eyebrow")),
            title: Some(email_i18n::text("email.hostFound.title")),
            body: email_i18n::text_with("email.hostFound.body", &vars),
            cta: Some(ModuleEmailCta {
                label: email_text::cta_label(
                    description.truncated,
                    email_i18n::text("email.hostFound.cta"),
                ),
                url: None,
                portaki_action: Some("open-module:lost-found:default".into()),
            }),
        },
        stay_id: Some(stay_id),
        // The invocation's property: the platform can refuse a stay that is not on it.
        property_id: Some(property_id),
        action_url: None,
    })
}

/// J+2 checkout follow-up — only when at least one declaration exists for the stay.
pub fn send_checkout_follow_up(ctx: &Context) -> Result<()> {
    let stay_id = ctx
        .guest
        .as_ref()
        .map(|guest| guest.session_id)
        .ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))?;

    let reports = storage::list_by_stay(stay_id)?;
    if reports.is_empty() {
        return Ok(());
    }

    let Some(joined) = join_descriptions(&reports) else {
        return Ok(());
    };

    let vars = [("description", joined.text.as_str())];
    email::send(&SendEmailArgs {
        email_id: "checkout-j2".into(),
        audience: EmailAudience::Guest,
        content: ModuleEmailSdui {
            subject: email_i18n::text("email.checkoutFollowUp.subject"),
            eyebrow: Some(email_i18n::text("email.checkoutFollowUp.eyebrow")),
            title: Some(email_i18n::text("email.checkoutFollowUp.title")),
            body: email_i18n::text_with("email.checkoutFollowUp.body", &vars),
            cta: Some(ModuleEmailCta {
                label: email_text::cta_label(
                    joined.truncated,
                    email_i18n::text("email.checkoutFollowUp.cta"),
                ),
                url: None,
                portaki_action: Some("open-module:lost-found:default".into()),
            }),
        },
        stay_id: Some(stay_id),
        property_id: None,
        action_url: None,
    })
}

/// The stay's declarations, each quoted, joined, and bounded as a whole.
fn join_descriptions(reports: &[LostFoundReport]) -> Option<Quoted> {
    let parts: Vec<Quoted> = reports
        .iter()
        .map(|row| description::to_plain_text(&row.item_description))
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .map(|text| email_text::quote_guest_text(&text))
        .collect();
    if parts.is_empty() {
        return None;
    }
    let joined = parts
        .iter()
        .map(|part| part.text.as_str())
        .collect::<Vec<_>>()
        .join(" · ");
    let bounded = email_text::clip_chars(&joined, JOINED_DESCRIPTIONS_MAX_CHARS);
    Some(Quoted {
        truncated: bounded.truncated || parts.iter().any(|part| part.truncated),
        text: bounded.text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_found_payload_includes_de_and_falls_back() {
        let subject = email_i18n::text("email.hostFound.subject");
        assert!(subject.translations.contains_key("de"));
        assert_eq!(
            subject.resolve("de"),
            subject.translations.get("de").map(String::as_str).unwrap()
        );
        assert!(!subject.resolve("xx").is_empty());
        let body = email_i18n::text_with("email.hostFound.body", &[("description", "scarf")]);
        assert!(body.resolve("en").contains("scarf"));
        assert!(body.resolve("fr").contains("scarf"));
    }
}
