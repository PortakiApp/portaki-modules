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
    // La page publique n'empêche jamais de publier le livret : hors bornes, son bloc se masque
    // (moins de trois) ou se coupe (plus de six), et l'hôte en est averti.
    let chosen = config.public_chosen().len();
    if config.public_enabled
        && !(crate::config::PUBLIC_MIN..=crate::config::PUBLIC_MAX).contains(&chosen)
    {
        items.push(PublishCheck {
            id: "config.public_enabled".into(),
            level: PublishLevel::Recommended,
            ok: false,
            label: crate::i18n::text("host.public.title"),
            hint: crate::i18n::text("publish.public.hint"),
        });
    }
    Ok(PublishReadiness { items })
}

/// Le libellé du champ en défaut : celui du formulaire, sans l'index de la ligne.
fn field_label(field: &str) -> &'static str {
    let last = field.rsplit('.').next().unwrap_or_default();
    if field.starts_with("host_activities") {
        return match last {
            "provider" => "host.hostActivities.provider",
            "duration" => "host.hostActivities.duration",
            "phone" => "host.hostActivities.phone",
            "url" => "host.hostActivities.url",
            _ => "host.hostActivities.title",
        };
    }
    match last {
        "title" => "host.spot.name",
        "category" => "host.spot.category",
        "warning" => "host.spot.warning",
        "season_from" => "host.spot.seasonFrom",
        "perk" => "host.spot.perk",
        "detail" => "host.spot.description",
        "price" => "host.spot.price",
        "parking" => "host.spot.parking",
        "phone" => "host.spot.phone",
        "url" => "host.spot.url",
        _ => "host.spots.title",
    }
}
