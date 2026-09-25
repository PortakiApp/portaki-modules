//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use portaki_test_utils::MockContextBuilder;
use serde_json::json;
use waste_recycling as _;

/// Les bacs d'une commune type, ceux des aperçus du catalogue.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    builder.with_config(&json!({
        "bins": [
            {
                "id": "jaune",
                "title": { "fr": "Bac jaune", "en": "Yellow bin" },
                "items": [
                    { "fr": "Emballages plastique et métal", "en": "Plastic and metal packaging" },
                    { "fr": "Cartons et papiers", "en": "Cardboard and paper" }
                ],
                "color": "yellow"
            },
            {
                "id": "verre",
                "title": { "fr": "Colonne à verre", "en": "Glass bank" },
                "items": [{ "fr": "Bouteilles et bocaux, sans bouchon", "en": "Bottles and jars, lids off" }],
                "color": "green"
            },
            {
                "id": "biodechets",
                "title": { "fr": "Bac à biodéchets", "en": "Food waste bin" },
                "items": [{ "fr": "Épluchures, marc de café, restes de repas", "en": "Peelings, coffee grounds, leftovers" }],
                "color": "brown"
            },
            {
                "id": "ordures",
                "title": { "fr": "Ordures ménagères", "en": "General waste" },
                "items": [{ "fr": "Tout le reste, en sac fermé", "en": "Everything else, in a closed bag" }],
                "color": "grey"
            }
        ],
        "collection_schedule": {
            "fr": "Bacs à sortir la veille au soir : mardi pour le jaune, vendredi pour les ordures ménagères.",
            "en": "Put the bins out the evening before: Tuesday for yellow, Friday for general waste."
        }
    }))
}

#[test]
fn every_surface_holds_on_every_case() {
    scenarios::check_surfaces(env!("CARGO_MANIFEST_DIR"), setup);
}

#[test]
fn every_example_runs() {
    scenarios::check_examples(concat!(env!("OUT_DIR"), "/portaki-emissions"), setup, &[]);
}
