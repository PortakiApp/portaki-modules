//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use portaki_sdk::capability;
use sections::{render_explore_sheet, render_home_card, reset_test_store};
use serde_json::json;
use serial_test::serial;

/// Le mot que l'hôte écrit à ses voyageurs, sur le logement d'exemple du livret.
fn sample_config() -> serde_json::Value {
    json!({
        "show_from": "booking",
        "sections": [{
            "id": "welcome",
            "title": { "fr": "Bienvenue à L'Islette", "en": "Welcome to L'Islette" },
            "body": {
                "fr": "Nous sommes heureux de vous accueillir.\n\nFaites comme chez vous : le café est dans le placard au-dessus de la machine, et les serviettes de plage sont dans le coffre de l'entrée.",
                "en": "We are glad to have you.\n\nMake yourself at home: the coffee is in the cupboard above the machine, and the beach towels are in the hallway chest."
            }
        }]
    })
}

/// Aucune surface n'a de chemin : le module n'entre pas au catalogue des aperçus, mais le livret
/// montre son mot dès l'accueil — c'est donc la démo qui en a besoin.
#[test]
#[serial]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    reset_test_store();
    let context = previews::guest(root)
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config());
    let (card, sheet) = context.run(|ctx| {
        (
            render_home_card(ctx.clone()).expect("card"),
            render_explore_sheet(ctx).expect("sheet"),
        )
    });
    previews::check_all(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        Vec::new(),
        vec![("home.card", card), ("explore.sheet", sheet)],
        None,
    );
}
