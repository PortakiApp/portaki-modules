//! Publication readiness: the field errors of [`ModuleConfig::problems`], each naming its field,
//! and a warning for an event without a pin (spec Événements §2.2).

use portaki_sdk::config::check;
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
            // Un rayon hors bornes avertit sans bloquer : l'ancien sélecteur offrait 60 et 100 km,
            // et ces hôtes ne doivent pas se retrouver bloqués à la publication suivante. La
            // recherche le borne déjà à 50.
            level: if field == "radius_km" {
                PublishLevel::Recommended
            } else {
                PublishLevel::Required
            },
            id: format!("config.{field}"),
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
    // Plus loin que le rayon (§2.2) : un avertissement, comme l'épingle manquante — le rayon
    // borne la recherche automatique, il n'invalide pas un lieu que l'hôte a choisi.
    let radius = f64::from(config.normalized_radius_km());
    if let Some(home) = ctx.property.coordinates.as_ref() {
        if let Some(index) = config.events.iter().position(|event| {
            !event.is_blank()
                && event.has_coords()
                && !check::within_km(
                    event.lat.unwrap_or_default(),
                    event.lng.unwrap_or_default(),
                    home.lat,
                    home.lng,
                    radius,
                )
        }) {
            items.push(PublishCheck {
                id: format!("config.events.{index}.lat"),
                level: PublishLevel::Recommended,
                ok: false,
                label: crate::i18n::text("host.event.where"),
                hint: crate::i18n::text("host.event.where.tooFar"),
            });
        }
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
        "tips" => "host.event.tips",
        "radius_km" => "host.nearby.radius",
        _ => "host.events.title",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::{MockContext, Property};
    use serde_json::json;

    /// Un lieu au-delà du rayon avertit, sans bloquer (§2.2) ; un rayon hors de 1 à 50 aussi.
    #[test]
    #[serial_test::serial]
    fn a_place_beyond_the_radius_warns() {
        // Le logement par défaut est à Cannes (43.55, 7.01) ; Nice est à ~25 km.
        let config = json!({
            "radius_km": 15,
            "events": [
                { "title": "Marché", "lat": 43.56, "lng": 7.02 },
                { "title": "Carnaval", "lat": 43.70, "lng": 7.27 }
            ]
        });
        MockContext::host()
            .with_property(Property::default())
            .with_config(&config)
            .run(|ctx| {
                let items = publish_readiness(ctx).unwrap().items;
                assert_eq!(items.len(), 1, "{items:?}");
                assert_eq!(items[0].id, "config.events.1.lat");
                assert_eq!(items[0].level, PublishLevel::Recommended);
                assert_eq!(
                    items[0].hint.get("fr"),
                    "Ce lieu est plus loin que le rayon choisi."
                );
            });
        let wide = json!({ "radius_km": 60, "events": config["events"] });
        MockContext::host()
            .with_property(Property::default())
            .with_config(&wide)
            .run(|ctx| {
                let items = publish_readiness(ctx).unwrap().items;
                assert_eq!(items.len(), 1, "{items:?}");
                assert_eq!(items[0].id, "config.radius_km");
                assert_eq!(items[0].level, PublishLevel::Recommended);
                assert_eq!(items[0].hint.get("fr"), "Entre 1 et 50 km.");
            });
    }
}
