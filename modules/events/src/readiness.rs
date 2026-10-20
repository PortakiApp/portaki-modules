//! Publication readiness: the field errors of [`ModuleConfig::problems`], each naming its field
//! and blocking, then the warnings (spec Événements §2.2).

use portaki_sdk::bundle_text;
use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    let home = ctx
        .property
        .coordinates
        .as_ref()
        .map(|point| (point.lat, point.lng));
    let check = |level: PublishLevel| {
        move |(field, hint): (String, portaki_sdk::contracts::i18n::I18nText)| PublishCheck {
            label: bundle_text!(field_label(&field)),
            level,
            id: format!("config.{field}"),
            ok: false,
            hint,
        }
    };
    let items = config
        .problems(home)
        .into_iter()
        .map(check(PublishLevel::Required))
        .chain(
            config
                .warnings()
                .into_iter()
                .map(check(PublishLevel::Recommended)),
        )
        .collect();
    Ok(PublishReadiness { items })
}

/// Le libellé du champ en défaut : celui du formulaire, sans l'index de la ligne.
fn field_label(field: &str) -> &'static str {
    match field.rsplit('.').next().unwrap_or_default() {
        "title" => "host.event.title",
        "end_time" => "host.event.endsAt",
        "lat" => "host.event.where",
        "url" => "host.event.url",
        "price" => "host.event.price",
        "access" => "host.event.access",
        "note" => "host.event.note",
        "tips" => "host.event.tips",
        "weekdays" => "host.event.weekdays",
        "dates" => "host.event.dates",
        "radius_km" => "host.nearby.radius",
        _ => "host.events.title",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::{MockContext, Property};
    use serde_json::json;

    /// Un lieu au-delà du rayon bloque (§2.2) ; un rayon hors de 1 à 50 aussi.
    #[test]
    #[serial_test::serial]
    fn a_place_beyond_the_radius_blocks() {
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
                assert_eq!(items[0].level, PublishLevel::Required);
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
                assert_eq!(items[0].level, PublishLevel::Required);
                assert_eq!(items[0].hint.get("fr"), "Entre 1 et 50 km.");
            });
    }

    /// Les jours manquants et le sixième conseil avertissent, sans bloquer la publication.
    #[test]
    #[serial_test::serial]
    fn missing_days_and_too_many_tips_are_recommended() {
        let config = json!({
            "events": [
                { "title": "Marché", "recurrence": "weekly", "lat": 43.55, "lng": 7.01 },
                { "title": "Concert", "tips": "1\n2\n3\n4\n5\n6", "lat": 43.55, "lng": 7.01 }
            ]
        });
        MockContext::host()
            .with_property(Property::default())
            .with_config(&config)
            .run(|ctx| {
                let items = publish_readiness(ctx).unwrap().items;
                let ids: Vec<(&str, PublishLevel)> =
                    items.iter().map(|i| (i.id.as_str(), i.level)).collect();
                assert_eq!(
                    ids,
                    [
                        ("config.events.0.weekdays", PublishLevel::Recommended),
                        ("config.events.1.tips", PublishLevel::Recommended)
                    ]
                );
                assert_eq!(items[0].hint.get("fr"), "Choisissez au moins un jour.");
                assert_eq!(items[0].label.get("fr"), "Jours");
            });
    }
}
