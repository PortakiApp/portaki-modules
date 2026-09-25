//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use consumables::seed_defaults;
use portaki_sdk::prelude::EmptyArgs;
use portaki_test_utils::MockContextBuilder;

/// Le catalogue par défaut, celui qu'un hôte obtient en un clic : sans lui, le livret n'a rien
/// à proposer. Le magasin en mémoire est par fil ; `seedDefaults` ne remplit qu'un catalogue vide.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    builder
        .clone()
        .run(|ctx| seed_defaults(ctx, EmptyArgs::default()))
        .expect("seed defaults");
    builder
}

#[test]
fn every_surface_holds_on_every_case() {
    scenarios::check_surfaces(env!("CARGO_MANIFEST_DIR"), setup);
}

#[test]
fn every_example_runs() {
    scenarios::check_examples(
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        setup,
        // Un signalement existant : un compte neuf n'en a aucun à passer « réassorti ».
        &["updateStatus"],
    );
}
