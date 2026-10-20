//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

mod sample;

#[path = "../../../support/scenarios.rs"]
mod scenarios;
use sections as _; // lie la crate : ses déclarations sont ce que les scénarios parcourent

#[test]
fn every_surface_holds_on_every_case() {
    scenarios::check_surfaces(env!("CARGO_MANIFEST_DIR"), sample::seeded_two);
}

#[test]
fn every_example_runs() {
    scenarios::check_examples(
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        sample::seeded_two,
        &[],
    );
}
