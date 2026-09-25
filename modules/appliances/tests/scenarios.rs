//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use appliances::{reset_test_store, save_appliance, ApplianceStatus, SaveApplianceArgs};
use portaki_test_utils::MockContextBuilder;

/// Deux appareils, dont les plaques mises en avant : le livret a de quoi montrer.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    reset_test_store();
    builder.clone().run(|ctx| {
        let devices = [
            ("plaques", "Plaques à induction", "🍳", true, "Cuisine"),
            ("lave-linge", "Lave-linge", "🌀", false, "Salle de bain"),
        ];
        for (order, (id, name, emoji, featured, location)) in devices.into_iter().enumerate() {
            let args = SaveApplianceArgs {
                id: Some(id.into()),
                name: name.into(),
                emoji: emoji.into(),
                description: "Appuyez 2 secondes sur la touche marche.".into(),
                featured,
                order: Some(order as i32),
                location: location.into(),
                manual_url: String::new(),
                safety_note: String::new(),
                status: ApplianceStatus::Active,
            };
            save_appliance(ctx.clone(), args).expect("save appliance");
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
        // Des identifiants d'appareils qu'un compte neuf n'a pas.
        &["deleteAppliance", "reorderAppliances"],
    );
}
