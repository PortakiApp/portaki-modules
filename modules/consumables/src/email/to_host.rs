//! A guest asked: the e-mail to the host.

use portaki_sdk::host::email::{
    self, clip, quote_guest, EmailAudience, EmailBlock, EmailPair, LocalizedEmailText,
    ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::prelude::*;
use uuid::Uuid;

use super::{cta, ITEM_ROW_MAX_CHARS};
use crate::{Entry, Level};

/// Guest shortage request → notify workspace owner.
pub fn to_host(property_id: Uuid, entry: &Entry) -> Result<()> {
    let request = &entry.item;
    let (level_fr, level_en) = match request.level {
        Level::Low => ("bientôt vide", "running low"),
        Level::Missing => ("manque", "missing"),
    };

    // A block line holds 200 chars: the item is cut shorter than a body quote.
    let (item, item_cut) = clip(&request.item_label, ITEM_ROW_MAX_CHARS);
    let note = Some(request.note.trim())
        .filter(|n| !n.is_empty())
        .map(quote_guest);
    let truncated = item_cut || note.as_ref().is_some_and(|(_, cut)| *cut);

    let mut body_fr = format!("Un voyageur signale un consommable ({level_fr}).");
    let mut body_en = format!("A guest reported a consumable ({level_en}).");
    if let Some((extra, _)) = &note {
        body_fr.push_str("\n\nPrécision : ");
        body_fr.push_str(extra);
        body_en.push_str("\n\nNote: ");
        body_en.push_str(extra);
    }
    let blocks = vec![EmailBlock::rows([
        EmailPair::new(
            LocalizedEmailText::new("Article", "Item"),
            LocalizedEmailText::both(item),
        ),
        EmailPair::new(
            LocalizedEmailText::new("Niveau", "Level"),
            LocalizedEmailText::new(level_fr, level_en),
        ),
    ])];

    email::send(&SendEmailArgs {
        email_id: format!("submitted-{}", entry.id),
        audience: EmailAudience::Host,
        content: ModuleEmailSdui {
            subject: LocalizedEmailText::new(
                "Un voyageur signale un consommable manquant",
                "A guest reported a missing consumable",
            ),
            eyebrow: Some(LocalizedEmailText::new("Consommables", "Consumables")),
            title: Some(LocalizedEmailText::new(
                "Nouveau signalement stock",
                "New stock report",
            )),
            body: LocalizedEmailText::new(body_fr, body_en),
            // No URL: with `property_id` set, the platform links the property page.
            cta: Some(cta(
                truncated,
                LocalizedEmailText::new("Voir le logement", "View property"),
                None,
            )),
            blocks,
            ..Default::default()
        },
        stay_id: entry.stay_id,
        property_id: Some(property_id),
        action_url: None,
    })
}
