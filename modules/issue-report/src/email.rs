//! Les deux e-mails : à l'hôte quand un voyageur signale, au voyageur quand l'hôte répond.

use portaki_sdk::bundle_text;
use portaki_sdk::host::email::{
    self, quote_guest, EmailAudience, EmailBlock, EmailPair, EmailTone, LocalizedEmailText,
    ModuleEmailCta, ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::prelude::*;
use portaki_sdk::resource::{Actor, Choice, Entry};

use crate::{Report, Status};

/// Un voyageur a signalé : le texte cité, la catégorie, la photo s'il y en a une.
pub fn to_host(entry: &Entry<Report, Status>) -> Result<()> {
    if entry.actor != Actor::Guest {
        return Ok(());
    }
    let report = &entry.item;
    let (summary, cut) = quote_guest(&report.summary);
    let (details, cut_details) = quote_guest(&report.details);
    let body = [summary, details]
        .into_iter()
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    let category: LocalizedEmailText = bundle_text!(&report.category.bundle_key()).into();
    let mut blocks = vec![EmailBlock::rows([EmailPair::new(
        LocalizedEmailText::new("Catégorie", "Category"),
        category,
    )])];
    if !report.photo.is_empty() {
        blocks.push(EmailBlock::note(
            EmailTone::Info,
            LocalizedEmailText::new(
                "Une photo est jointe — visible dans le tableau de bord.",
                "A photo is attached — see it in the dashboard.",
            ),
        ));
    }
    send(
        entry,
        EmailAudience::Host,
        format!("submitted-{}", entry.id),
        ModuleEmailSdui {
            subject: LocalizedEmailText::new(
                "Un voyageur a signalé un problème",
                "A guest reported an issue",
            ),
            eyebrow: Some(LocalizedEmailText::both("Signalement")),
            title: Some(LocalizedEmailText::new(
                "Nouveau problème signalé",
                "New issue report",
            )),
            body: LocalizedEmailText::both(body),
            cta: Some(cta(
                cut || cut_details,
                LocalizedEmailText::new("Voir le logement", "View property"),
                None,
            )),
            blocks,
            ..Default::default()
        },
    )
}

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

fn cta(cut: bool, own: LocalizedEmailText, action: Option<&str>) -> ModuleEmailCta {
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

fn send(
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
