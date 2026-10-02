//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use portaki_test_utils::MockContextBuilder;
use safety_shutoffs as _;
use serde_json::json;

/// Les organes d'un gîte type, ceux des aperçus du catalogue.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    builder.with_config(&sample_config())
}

pub fn sample_config() -> serde_json::Value {
    json!({
        "shutoffs": [
            {
                "id": "tableau",
                "kind": "electricity",
                "title": { "fr": "Tableau électrique", "en": "Fuse box" },
                "location": { "fr": "Placard à droite de l’entrée", "en": "Cupboard to the right of the entrance" },
                "instruction": {
                    "fr": "Disjoncteur principal en haut à gauche : abaissez-le pour tout couper.",
                    "en": "Main breaker, top left: flip it down to cut everything."
                }
            },
            {
                "id": "eau",
                "kind": "water",
                "title": { "fr": "Vannes d’arrêt d’eau", "en": "Water shutoff valves" },
                "location": { "fr": "Trappe au plafond des WC", "en": "Hatch in the toilet ceiling" },
                "instruction": {
                    "fr": "Eau froide à gauche, eau chaude à droite. Tournez dans le sens des aiguilles d’une montre.",
                    "en": "Cold water left, hot water right. Turn clockwise."
                }
            },
            {
                "id": "gaz",
                "kind": "gas",
                "title": { "fr": "Robinet de gaz", "en": "Gas valve" },
                "location": { "fr": "Sous l’évier de la cuisine", "en": "Under the kitchen sink" },
                "instruction": {
                    "fr": "Quart de tour : poignée en travers du tuyau = fermé.",
                    "en": "Quarter turn: handle across the pipe means closed."
                }
            },
            {
                "id": "extincteur",
                "kind": "extinguisher",
                "title": { "fr": "Extincteur", "en": "Fire extinguisher" },
                "location": { "fr": "Derrière la porte du cellier", "en": "Behind the pantry door" }
            },
            {
                "id": "detecteur",
                "kind": "smoke_detector",
                "title": { "fr": "Détecteur de fumée", "en": "Smoke detector" },
                "location": { "fr": "Plafond du couloir, à l’étage", "en": "Upstairs hallway ceiling" },
                "instruction": {
                    "fr": "S’il bipe toutes les minutes, la pile est faible : prévenez-nous.",
                    "en": "If it beeps every minute the battery is low: let us know."
                }
            }
        ],
        "general_note": {
            "fr": "En cas de fuite d’eau, coupez d’abord la vanne, puis appelez-moi à toute heure.",
            "en": "If water leaks, shut the valve first, then call me at any hour."
        }
    })
}

#[test]
fn every_surface_holds_on_every_case() {
    scenarios::check_surfaces(env!("CARGO_MANIFEST_DIR"), setup);
}

#[test]
fn every_example_runs() {
    scenarios::check_examples(concat!(env!("OUT_DIR"), "/portaki-emissions"), setup, &[]);
}
