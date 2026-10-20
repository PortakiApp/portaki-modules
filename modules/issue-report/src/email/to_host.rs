//! Un voyageur a signalé : l'e-mail à l'hôte.

use portaki_sdk::bundle_text;
use portaki_sdk::host::email::{
    quote_guest, EmailAudience, EmailBlock, EmailPair, EmailTone, LocalizedEmailText,
    ModuleEmailSdui,
};
use portaki_sdk::prelude::*;
use portaki_sdk::resource::{Actor, Choice, Entry};

use super::{cta, send};
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
