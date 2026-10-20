//! Les deux e-mails, un par fichier ; le bouton et l'envoi sont communs.

use portaki_sdk::bundle_text;
use portaki_sdk::host::email::{
    self, EmailAudience, LocalizedEmailText, ModuleEmailCta, ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::prelude::*;
use portaki_sdk::resource::Entry;

use crate::{Report, Status};

pub mod to_guest;
pub mod to_host;

pub(crate) fn cta(cut: bool, own: LocalizedEmailText, action: Option<&str>) -> ModuleEmailCta {
    ModuleEmailCta {
        label: if cut {
            bundle_text!("email.cta.seeMore").into()
        } else {
            own
        },
        url: None,
        portaki_action: action.map(str::to_string),
    }
}

pub(crate) fn send(
    entry: &Entry<Report, Status>,
    audience: EmailAudience,
    email_id: String,
    content: ModuleEmailSdui,
) -> Result<()> {
    email::send(&SendEmailArgs {
        email_id,
        audience,
        content,
        stay_id: entry.stay_id,
        property_id: None,
        action_url: None,
    })
}
