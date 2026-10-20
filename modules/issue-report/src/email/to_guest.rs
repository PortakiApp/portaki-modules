//! L'hôte a répondu : l'e-mail au voyageur.

use portaki_sdk::bundle_text;
use portaki_sdk::host::email::{self, quote_guest, EmailAudience, ModuleEmailSdui};
use portaki_sdk::prelude::*;
use portaki_sdk::resource::Entry;

use super::{cta, send};
use crate::{Report, Status};

/// L'hôte a répondu : sa réponse, et le signalement qu'elle concerne.
pub fn to_guest(entry: &Entry<Report, Status>, text: &str) -> Result<()> {
    let (reply, cut) = quote_guest(text);
    let (summary, _) = email::clip(&entry.item.summary, 120);
    let vars = [("summary", summary.as_str()), ("reply", reply.as_str())];
    send(
        entry,
        EmailAudience::Guest,
        format!("host-reply-{}-{}", entry.id, entry.history.len()),
        ModuleEmailSdui {
            subject: bundle_text!("email.hostReply.subject").into(),
            eyebrow: Some(bundle_text!("email.hostReply.eyebrow").into()),
            title: Some(bundle_text!("email.hostReply.title", &vars).into()),
            body: bundle_text!("email.hostReply.body", &vars).into(),
            cta: Some(cta(
                cut,
                bundle_text!("email.hostReply.cta").into(),
                Some("open-module:issue-report:default"),
            )),
            ..Default::default()
        },
    )
}
