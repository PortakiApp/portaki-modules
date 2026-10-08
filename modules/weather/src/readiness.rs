//! `publishReadiness` — the rules of the settings drawer, checked again before « Publier ».
//!
//! The same functions put the error under the field (`host`) and here, so the drawer and the
//! publication say the same thing. A point is listed only when it is not met.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;
use crate::i18n::text;

#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    Ok(PublishReadiness {
        items: checks(&config, ctx.property.coordinates),
    })
}

fn checks(
    config: &ModuleConfig,
    property: Option<portaki_sdk::sdui::common::GeoPoint>,
) -> Vec<PublishCheck> {
    let mut items = Vec::new();
    if let Some(hint) = config.label_error() {
        items.push(point(
            "config.location_label",
            PublishLevel::Required,
            "host.locationLabel.label",
            hint,
        ));
    }
    if let Some(hint) = config.pin_error() {
        items.push(point(
            "config.location_lat",
            PublishLevel::Required,
            "host.locationOverride.position",
            hint,
        ));
    }
    if config.override_is_far(property) {
        items.push(point(
            "config.location_lat",
            PublishLevel::Recommended,
            "host.locationOverride.position",
            text("host.locationOverride.far"),
        ));
    }
    // Cas 4 : ni adresse géocodée ni position corrigée, pas de prévisions — le module reste
    // publiable, le voyageur voit « pas de lieu ».
    if config.point(property).is_none() && config.pin_error().is_none() {
        items.push(point(
            "config.location_override",
            PublishLevel::Recommended,
            "host.location.label",
            text("host.location.missing"),
        ));
    }
    items
}

fn point(
    id: &str,
    level: PublishLevel,
    label: &str,
    hint: portaki_sdk::contracts::i18n::I18nText,
) -> PublishCheck {
    PublishCheck {
        id: id.into(),
        level,
        ok: false,
        label: text(label),
        hint,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_sdk::sdui::common::GeoPoint;

    const HOME: GeoPoint = GeoPoint {
        lat: 47.25,
        lng: 0.43,
    };

    fn ids(items: &[PublishCheck]) -> Vec<(&str, PublishLevel)> {
        items
            .iter()
            .map(|item| (item.id.as_str(), item.level))
            .collect()
    }

    #[test]
    fn a_default_config_at_a_geocoded_property_is_ready() {
        assert!(checks(&ModuleConfig::default(), Some(HOME)).is_empty());
    }

    #[test]
    fn a_long_label_or_a_missing_pin_blocks() {
        let config = ModuleConfig {
            location_label: "x".repeat(41),
            location_override: true,
            ..ModuleConfig::default()
        };
        let items = checks(&config, Some(HOME));
        assert_eq!(
            ids(&items),
            [
                ("config.location_label", PublishLevel::Required),
                ("config.location_lat", PublishLevel::Required)
            ]
        );
        assert_eq!(items[0].hint.get("fr"), "40 caractères au maximum.");
        assert_eq!(items[1].hint.get("fr"), "Placez l'épingle sur la carte.");
        assert!(
            !items[0].label.get("fr").starts_with("host."),
            "libellé traduit, pas sa clé"
        );
    }

    #[test]
    fn a_far_pin_or_no_position_only_warns() {
        let far = ModuleConfig {
            location_override: true,
            location_lat: Some(45.12),
            location_lng: Some(5.88),
            ..ModuleConfig::default()
        };
        assert_eq!(
            ids(&checks(&far, Some(HOME))),
            [("config.location_lat", PublishLevel::Recommended)]
        );
        assert_eq!(
            ids(&checks(&ModuleConfig::default(), None)),
            [("config.location_override", PublishLevel::Recommended)]
        );
    }
}
