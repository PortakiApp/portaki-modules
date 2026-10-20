//! Module-owned transactional emails via `host::email::send`.
//!
//! Guest-typed text is quoted within a fixed length — the report keeps it whole; see
//! [`crate::email_text`].

use portaki_sdk::host::email::{
    self, EmailAudience, ModuleEmailCta, ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::email_text::{self, ITEM_ROW_MAX_CHARS};
use crate::entities::ConsumableReport;

/// The host's reply → the guest of the request's stay. The platform finds the address from the
/// stay; a guest without one simply gets nothing (`requires_guest_email`).
pub fn reply_to_guest(report: &ConsumableReport, reply: &str) -> Result<()> {
    let reply = email_text::quote_guest_text(reply);
    let item = email_text::clip_chars(&report.item_label, ITEM_ROW_MAX_CHARS);
    let vars = [("item", item.text.as_str()), ("reply", reply.text.as_str())];
    email::send(&SendEmailArgs {
        // A fresh suffix per reply: a second answer is a second e-mail, not a duplicate.
        email_id: format!("host-reply-{}", Uuid::new_v4()),
        audience: EmailAudience::Guest,
        content: ModuleEmailSdui {
            subject: crate::email_i18n::text_with("email.hostReply.subject", &vars),
            eyebrow: Some(crate::email_i18n::text("email.hostReply.eyebrow")),
            title: Some(crate::email_i18n::text_with("email.hostReply.title", &vars)),
            body: crate::email_i18n::text_with("email.hostReply.body", &vars),
            cta: Some(ModuleEmailCta {
                label: email_text::cta_label(
                    reply.truncated,
                    crate::email_i18n::text("email.hostReply.cta"),
                ),
                url: None,
                portaki_action: Some("open-module:consumables:default".into()),
            }),
            ..Default::default()
        },
        stay_id: Some(report.stay_id),
        property_id: None,
        action_url: None,
    })
}
