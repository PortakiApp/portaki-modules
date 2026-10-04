//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use portaki_test_utils::MockContextBuilder;
use train as _;

/// La gare de l'hôte et un fournisseur qui répond : les cas pathologiques portent sur le rendu,
/// pas sur l'absence de données, que `integration.rs` couvre à part.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    builder
        .with_config(&serde_json::json!({ "station": "Antibes" }))
        .with_connector_response(
            "sncf",
            "find_place",
            include_str!("fixtures/sncf-places.json"),
        )
        .with_connector_response(
            "sncf",
            "departures",
            include_str!("fixtures/sncf-departures.json"),
        )
        .with_connector_response(
            "sncf",
            "arrivals",
            include_str!("fixtures/sncf-departures.json"),
        )
}

#[test]
fn every_surface_holds_on_every_case() {
    scenarios::check_surfaces(env!("CARGO_MANIFEST_DIR"), setup);
}

#[test]
fn every_example_runs() {
    scenarios::check_examples(concat!(env!("OUT_DIR"), "/portaki-emissions"), setup, &[]);
}
