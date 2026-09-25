//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use portaki_test_utils::MockContextBuilder;
use rules::{reset_test_store, save_content, SaveContentArgs};
use serde_json::json;

/// Le règlement type d'une location de vacances.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    reset_test_store();
    builder.clone().run(|ctx| {
        let content_fr = json!({
            "items": [
                { "icon": "clock-circle", "title": "Calme après 22 h", "subtitle": "Merci de penser au voisinage" },
                { "icon": "x", "title": "Logement non-fumeur", "subtitle": "" },
                { "icon": "check-circle", "title": "Animaux bienvenus", "subtitle": "Prévenez-nous avant votre arrivée" }
            ]
        });
        let args = SaveContentArgs {
            items: Vec::new(),
            content_fr: content_fr.to_string(),
            content_en: String::new(),
        };
        save_content(ctx, args).expect("save");
    });
    builder
}

#[test]
fn every_surface_holds_on_every_case() {
    scenarios::check_surfaces(env!("CARGO_MANIFEST_DIR"), setup);
}

#[test]
fn every_example_runs() {
    scenarios::check_examples(concat!(env!("OUT_DIR"), "/portaki-emissions"), setup, &[]);
}
