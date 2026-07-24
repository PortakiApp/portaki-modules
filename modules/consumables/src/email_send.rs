//! Module-owned transactional emails via `host::email::send`.

use portaki_sdk::host::email::{
    self, EmailAudience, LocalizedEmailText, ModuleEmailCta, ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::prelude::*;
use uuid::Uuid;

/// Guest shortage report → notify workspace owner.
pub fn notify_host_submitted(
    property_id: Uuid,
    stay_id: Uuid,
    report_id: Uuid,
    item_label: &str,
    level: &str,
    note: Option<&str>,
) -> Result<()> {
    let level_fr = match level {
        "low" => "bientôt vide",
        _ => "manque",
    };
    let level_en = match level {
        "low" => "running low",
        _ => "missing",
    };

    let mut body_fr = format!("Un voyageur signale un consommable ({level_fr}) :\n\n{item_label}");
    let mut body_en = format!("A guest reported a consumable ({level_en}):\n\n{item_label}");
    if let Some(extra) = note {
        body_fr.push_str("\n\nPrécision : ");
        body_fr.push_str(extra);
        body_en.push_str("\n\nNote: ");
        body_en.push_str(extra);
    }

    email::send(&SendEmailArgs {
        email_id: format!("submitted-{report_id}"),
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
            cta: Some(ModuleEmailCta {
                label: LocalizedEmailText::new("Voir le logement", "View property"),
                url: None,
                portaki_action: None,
            }),
        },
        stay_id: Some(stay_id),
        property_id: Some(property_id),
        action_url: None,
    })
}
