//! Publication readiness: the field errors of [`ModuleConfig::problems`], each naming its field,
//! and a warning for an address without a pin (spec Bons plans §2.1).

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
    // Sans position : ni repère, ni distance, ni itinéraire. La spec l'exige ; des adresses
    // existantes n'en ont pas et s'affichent quand même, d'où l'avertissement.
    if let Some(index) = config
        .spots
        .iter()
        .position(|spot| !spot.is_blank() && (spot.lat.is_none() || spot.lng.is_none()))
    {
        items.push(PublishCheck {
            id: format!("config.spots.{index}.lat"),
            level: PublishLevel::Recommended,
            ok: false,
            label: crate::i18n::text("host.spot.address"),
            hint: crate::i18n::text("host.spot.position.required"),
        });
    }
    Ok(PublishReadiness { items })
}

/// Le libellé du champ en défaut : celui du formulaire, sans l'index de la ligne.
fn field_label(field: &str) -> &'static str {
    match field.rsplit('.').next().unwrap_or_default() {
        "title" => "host.spot.name",
        "perk" => "host.spot.perk",
        "detail" => "host.spot.description",
        "price" => "host.spot.price",
        "parking" => "host.spot.parking",
        "phone" => "host.spot.phone",
        "url" => "host.spot.url",
        _ => "host.spots.title",
    }
}
