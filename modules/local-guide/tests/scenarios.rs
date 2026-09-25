//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use local_guide as _;
use portaki_sdk::capability;
use portaki_test_utils::MockContextBuilder;
use serde_json::json;
use serial_test::serial;

/// Trois adresses de l'hôte, situées, et la billetterie Tiqets (pool Portaki) contre sa
/// réponse enregistrée.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    builder
        .with_extra_capability_ids(&[capability::external::TIQETS_POOL.as_str()])
        .with_connector_response(
            "tiqets",
            "nearby_products",
            include_str!("fixtures/tiqets-products-nearby.json"),
        )
        .with_config(&json!({
            "spots": [
                {
                    "id": "boulangerie",
                    "title": { "fr": "La boulangerie du village", "en": "The village bakery" },
                    "category": "Boulangerie",
                    "distance": "350 m",
                    "detail": { "fr": "Croissants dès 7 h, fermée le lundi.", "en": "Croissants from 7am, closed on Mondays." },
                    "lat": 43.5531, "lng": 7.0152
                },
                {
                    "id": "marche",
                    "title": { "fr": "Marché provençal", "en": "Provençal market" },
                    "category": "Marché",
                    "distance": "1,2 km",
                    "tag": "Samedi matin",
                    "url": "https://www.cannes-destination.fr/marches",
                    "lat": 43.5552, "lng": 7.0214
                },
                {
                    "id": "lac",
                    "title": { "fr": "Lac de Saint-Cassien", "en": "Lake Saint-Cassien" },
                    "category": "Baignade",
                    "distance": "9 km",
                    "detail": { "fr": "Eau calme et plage de galets, idéal avec des enfants.", "en": "Calm water and a pebble beach, great with kids." }
                }
            ],
            "disclaimer": { "fr": "Suggestions de votre hôte, sans partenariat.", "en": "Your host's picks, no partnership." },
            "tiqets_enabled": true,
            "tiqets_radius_km": "20"
        }))
}

#[test]
#[serial]
fn every_surface_holds_on_every_case() {
    scenarios::check_surfaces(env!("CARGO_MANIFEST_DIR"), setup);
}

#[test]
#[serial]
fn every_example_runs() {
    scenarios::check_examples(concat!(env!("OUT_DIR"), "/portaki-emissions"), setup, &[]);
}
