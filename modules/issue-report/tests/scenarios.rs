//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use issue_report as _; // lie la crate : ses déclarations sont ce que les scénarios parcourent

fn setup(
    builder: portaki_test_utils::MockContextBuilder,
) -> portaki_test_utils::MockContextBuilder {
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
