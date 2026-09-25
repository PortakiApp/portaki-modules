//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use emergency_contacts as _;
use portaki_test_utils::MockContextBuilder;
use serde_json::json;

/// Des numéros utiles et la ligne de l'hôte, ceux des aperçus du catalogue.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    builder.with_config(&json!({
        "contacts": [
            { "id": "pharmacie", "label": { "fr": "Pharmacie de garde", "en": "On-call pharmacy" }, "phone": "3237", "note": { "fr": "Service national", "en": "National service" } },
            { "id": "medecin", "label": { "fr": "Médecin généraliste", "en": "General practitioner" }, "phone": "01 99 00 12 34", "note": { "fr": "Cabinet du centre, sur rendez-vous", "en": "Town centre practice, by appointment" } },
            { "id": "samu", "label": { "fr": "SAMU", "en": "Ambulance" }, "phone": "15" }
        ],
        "host_visible_phone": "06 39 98 12 34"
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
