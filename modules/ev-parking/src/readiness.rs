//! Publication readiness: the field errors of [`ModuleConfig::problems`], each naming its field.

use portaki_sdk::bundle_text;
use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    let items = config
        .problems()
        .into_iter()
        .map(|(field, error)| PublishCheck {
            id: format!("config.{field}"),
            level: PublishLevel::Required,
            ok: false,
            label: bundle_text!(match field {
                "spot_label" => "host.spotLabel.label",
                "parking_code" => "host.parkingCode.label",
                "power_kw" => "host.power.label",
                "charger_pin" => "host.chargerPin.label",
                "instructions" => "host.instructions.label",
                "price" => "host.price.label",
                "booking_note" => "host.bookingNote.label",
                other => unreachable!("no label for config field {other}"),
            }),
            hint: error,
        })
        .collect::<Vec<_>>();
    // Une épingle à plus de 2 km du logement : un avertissement, jamais un blocage (§2.1).
    let mut items = items;
    if config.far_from(ctx.property.coordinates.map(|c| (c.lat, c.lng))) {
        items.push(PublishCheck {
            id: "config.charger_lat".into(),
            level: PublishLevel::Recommended,
            ok: false,
            label: bundle_text!("host.position.label"),
            hint: bundle_text!("host.position.far"),
        });
    }
    Ok(PublishReadiness { items })
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::MockContext;
    use serde_json::json;

    /// Every field `problems` can name has its label, and blocks; the far pin only warns.
    #[test]
    #[serial_test::serial]
    fn every_problem_blocks_under_its_label_and_the_far_pin_warns() {
        let config = json!({
            "parking_code": "12",
            "power_kw": 1.0,
            "charger_pin": "1",
            "instructions": "x".repeat(281),
            "pricing": "per_kwh",
            "booking_required": true,
            "booking_note": "x".repeat(121),
            "charger_lat": 45.0,
            "charger_lng": 7.12
        });
        let items = MockContext::host()
            .with_config(&config)
            .with_coordinates(Some(portaki_sdk::sdui::GeoPoint::new(43.55, 7.12)))
            .run(|ctx| publish_readiness(ctx).unwrap().items);
        let ids: Vec<_> = items.iter().map(|i| (i.id.as_str(), i.level)).collect();
        use PublishLevel::{Recommended, Required};
        assert_eq!(
            ids,
            [
                ("config.spot_label", Required),
                ("config.parking_code", Required),
                ("config.power_kw", Required),
                ("config.charger_pin", Required),
                ("config.instructions", Required),
                ("config.price", Required),
                ("config.booking_note", Required),
                ("config.charger_lat", Recommended),
            ]
        );
    }
}
