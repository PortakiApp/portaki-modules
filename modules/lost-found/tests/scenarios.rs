//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use lost_found::{reset_test_store, submit, submit_found, SubmitArgs, SubmitFoundArgs};
use portaki_test_utils::MockContextBuilder;

/// Un objet déjà déclaré sur le séjour, pour que les cartes et la vue hôte montrent son suivi :
/// signalé par le voyageur côté livret, retrouvé par l'hôte côté tableau de bord. Le magasin en
/// mémoire est par fil : repartir de zéro à chaque contexte évite d'empiler les signalements.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    reset_test_store();
    let seeded = builder.clone().run(|ctx| {
        let stay_id = ctx.stay.as_ref().map(|stay| stay.stay_id);
        if ctx.guest.is_some() {
            submit(
                ctx,
                SubmitArgs {
                    kind: "lost".into(),
                    item_description: "Chargeur de téléphone blanc".into(),
                    contact_hint: None,
                    details: Some("Sans doute branché près du lit de la chambre 2.".into()),
                },
            )
        } else {
            submit_found(
                ctx,
                SubmitFoundArgs {
                    stay_ids: Vec::new(),
                    stay_id,
                    description: "Lunettes de soleil retrouvées sur la terrasse".into(),
                    status: None,
                },
            )
        }
    });
    seeded.expect("seed report");
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
        // Un signalement existant : un compte neuf n'en a aucun dont changer le statut.
        &["updateStatus"],
    );
}
