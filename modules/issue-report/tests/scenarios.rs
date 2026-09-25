//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use issue_report::{reset_test_store, submit, Category, SubmitArgs};
use portaki_test_utils::scenarios::get;
use portaki_test_utils::MockContextBuilder;

/// Un signalement déjà envoyé, pour que la carte et la tuile hôte montrent son suivi. Seul un
/// voyageur signale : côté hôte, celui du cas normal. Le magasin en mémoire est par fil :
/// repartir de zéro à chaque contexte évite d'empiler les signalements.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    reset_test_store();
    let reporter = if builder.context().guest.is_some() {
        builder.clone()
    } else {
        get("normal").guest()
    };
    reporter
        .run(|ctx| {
            submit(
                ctx,
                SubmitArgs {
                    category: Category::Appliance,
                    summary: "Le four ne chauffe plus".into(),
                    details: Some("Le voyant s'allume mais la température ne monte pas.".into()),
                    photo: None,
                },
            )
        })
        .expect("submit");
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
        // Un signalement existant : un compte neuf n'en a aucun à clore.
        &["resolve"],
    );
}
