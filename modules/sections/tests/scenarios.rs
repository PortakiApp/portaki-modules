//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use portaki_test_utils::MockContextBuilder;
use serde_json::json;

/// Deux sections, l'une traduite, l'autre en français seulement, le mot visible dès la
/// réservation.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    sections::reset_test_store();
    builder.with_config(&json!({
        "show_from": "booking",
        "sections": [
            { "id": "a", "title": { "fr": "Bienvenue", "en": "Welcome" },
              "body": { "fr": "Bienvenue à **L'Islette** !", "en": "Welcome to **L'Islette**!" } },
            { "id": "b", "title": "L'appartement", "body": "2 chambres, terrasse vue mer." }
        ]
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
