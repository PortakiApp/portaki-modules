//! The host replied: the e-mail to the guest.

use portaki_sdk::host::email::{
    self, clip, quote_guest, EmailAudience, ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::prelude::*;

use super::{cta, text, text_with, ITEM_ROW_MAX_CHARS};
use crate::Entry;

/// The host's reply → the guest of the request's stay. The platform finds the address from the
/// stay; a guest without one simply gets nothing (`requires_guest_email`).
pub fn to_guest(entry: &Entry, reply: &str) -> Result<()> {
    let (reply, cut) = quote_guest(reply);
    let (item, _) = clip(&entry.item.item_label, ITEM_ROW_MAX_CHARS);
    let vars = [("item", item.as_str()), ("reply", reply.as_str())];
    email::send(&SendEmailArgs {
        // One id per reply: a second answer is a second e-mail, not a duplicate.
        email_id: format!("host-reply-{}-{}", entry.id, entry.history.len()),
        audience: EmailAudience::Guest,
        content: ModuleEmailSdui {
            subject: text_with("email.hostReply.subject", &vars),
            eyebrow: Some(text("email.hostReply.eyebrow")),
            title: Some(text_with("email.hostReply.title", &vars)),
            body: text_with("email.hostReply.body", &vars),
            cta: Some(cta(
                cut,
                text("email.hostReply.cta"),
                Some("open-module:consumables:default"),
            )),
            ..Default::default()
        },
        stay_id: entry.stay_id,
        property_id: None,
        action_url: None,
    })
}
