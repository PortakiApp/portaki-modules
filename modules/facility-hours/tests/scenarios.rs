//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use facility_hours as _;
use portaki_test_utils::MockContextBuilder;
use serde_json::json;

/// Les équipements d'une résidence de vacances, ceux des aperçus du catalogue.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    builder.with_config(&json!({
        "facilities": [
            {
                "id": "piscine",
                "title": { "fr": "Piscine", "en": "Swimming pool" },
                "lines": {
                    "fr": "Tous les jours, de juin à septembre\nEnfants sous la surveillance d'un adulte",
                    "en": "Every day, June to September\nChildren must be supervised by an adult"
                },
                "hours": "9 h – 20 h",
                "note": { "fr": "Bonnet de bain non obligatoire.", "en": "Swim cap not required." }
            },
            {
                "id": "laverie",
                "title": { "fr": "Laverie", "en": "Laundry room" },
                "lines": { "fr": "Rez-de-chaussée, bâtiment B", "en": "Ground floor, building B" },
                "hours": "7 h – 22 h"
            },
            {
                "id": "accueil",
                "title": { "fr": "Accueil de la résidence", "en": "Reception" },
                "lines": { "fr": "Du lundi au samedi", "en": "Monday to Saturday" },
                "hours": "8 h 30 – 12 h · 14 h – 18 h"
            }
        ],
        "general_note": { "fr": "Horaires susceptibles de varier les jours fériés.", "en": "Hours may vary on public holidays." }
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
