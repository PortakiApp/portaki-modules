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
            { "id": "paulette", "label": { "fr": "Paulette, la voisine", "en": "Paulette, the neighbour" }, "phone": "+33639980012", "note": { "fr": "Elle a un double des clés.", "en": "She has a spare key." } },
            { "id": "plombier", "label": { "fr": "Plombier", "en": "Plumber" }, "phone": "+33199001234", "note": { "fr": "Pour une fuite d'eau", "en": "For a water leak" } }
        ],
        "host_visible_phone": "+33639981234",
        "host_availability": "hours",
        "host_hours": "08:00-21:00",
        "pharmacy": "Pharmacie du Port",
        "pharmacy_phone": "+33199005678",
        "pharmacy_hours": "mon=09:00-19:30;tue=09:00-19:30;wed=09:00-19:30;thu=09:00-19:30;fri=09:00-19:30;sat=09:00-19:30",
        "doctor": "Dr Le Gall",
        "doctor_phone": "+33199009012"
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
