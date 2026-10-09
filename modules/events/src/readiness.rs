//! Publication readiness: the field errors of [`ModuleConfig::problems`], each naming its field,
//! and a warning for an event without a pin (spec Événements §2.2).

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    let mut items: Vec<PublishCheck> = config
        .problems()
        .into_iter()
        .map(|(field, error)| PublishCheck {
            label: crate::i18n::text(field_label(&field)),
            id: format!("config.{field}"),
            level: PublishLevel::Required,
            ok: false,
            hint: error,
        })
        .collect();
    // Sans position, l'événement reste dans le livret mais n'a ni repère, ni itinéraire, ni
    // distance : la spec le demande, sans bloquer un hôte dont les événements n'en avaient pas.
    if let Some(index) = config
        .events
        .iter()
        .position(|event| !event.is_blank() && !event.has_coords())
    {
        items.push(PublishCheck {
            id: format!("config.events.{index}.lat"),
            level: PublishLevel::Recommended,
            ok: false,
            label: crate::i18n::text("host.event.where"),
            hint: crate::i18n::text("host.event.where.required"),
        });
    }
    Ok(PublishReadiness { items })
}

/// Le libellé du champ en défaut : celui du formulaire, sans l'index de la ligne.
fn field_label(field: &str) -> &'static str {
    match field.rsplit('.').next().unwrap_or_default() {
        "title" => "host.event.title",
        "ends_at" => "host.event.endsAt",
        "url" => "host.event.url",
        "price" => "host.event.price",
        "access" => "host.event.access",
        "note" => "host.event.note",
        _ => "host.events.title",
    }
}
