//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

mod sample;

use rules as _; // lie la crate : ses déclarations sont ce que les scénarios parcourent
#[path = "../../../support/scenarios.rs"]
mod scenarios;

#[test]
fn every_surface_holds_on_every_case() {
    scenarios::check_surfaces(env!("CARGO_MANIFEST_DIR"), sample::seeded);
}

#[test]
fn every_example_runs() {
    scenarios::check_examples(
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        sample::seeded,
        &[],
    );
}
