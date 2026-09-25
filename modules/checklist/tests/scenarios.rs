//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use checklist::{create_checklist, render_home_card, reset_test_store, CreateChecklistArgs};
use portaki_test_utils::scenarios::check_each;
use portaki_test_utils::MockContextBuilder;

/// La liste de départ du voyageur et le ménage de l'hôte, tels que les modèles les créent.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    reset_test_store();
    builder.clone().run(|ctx| {
        for template in ["departure", "cleaning"] {
            let args = CreateChecklistArgs {
                template: template.into(),
            };
            create_checklist(ctx.clone(), args).expect("create");
        }
    });
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
        &[
            // Une liste, un élément ou une tâche qu'un compte neuf n'a pas : chacune de ces
            // opérations vise un identifiant.
            "updateConfig",
            "deleteChecklist",
            "completeItem",
            "uncompleteItem",
            "taskToggle",
            "taskComplete",
        ],
    );
}

/// La liste de départ s'ouvre 48 h avant le départ (10 h) ; l'horloge des cas est figée à 10 h.
#[test]
fn the_departure_list_opens_two_days_before_check_out() {
    check_each(|scenario| {
        let surface = setup(scenario.guest())
            .run(render_home_card)
            .map_err(|e| e.to_string())?;
        let shown = serde_json::to_string(&surface)
            .unwrap()
            .contains("completeItem");
        let due = scenario.stay.check_out_offset <= 2;
        if shown == due {
            Ok(())
        } else {
            Err(format!(
                "list shown: {shown}, departure in {} day(s)",
                scenario.stay.check_out_offset
            ))
        }
    });
}
