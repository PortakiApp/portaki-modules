//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use safety_shutoffs::{render_explore_detail, render_home_card};
use serde_json::json;

/// Les organes d'un gîte type : les six lignes de la planche `Module Securite`.
fn sample_config() -> serde_json::Value {
    json!({
        "shutoffs": [
            {
                "id": "tableau",
                "kind": "electricity",
                "title": { "fr": "Tableau électrique", "en": "Fuse box" },
                "location": { "fr": "Placard à droite de l’entrée", "en": "Cupboard to the right of the entrance" },
                "instruction": {
                    "fr": "Disjoncteur principal en haut à gauche : abaissez-le pour tout couper.",
                    "en": "Main breaker, top left: flip it down to cut everything."
                }
            },
            {
                "id": "eau",
                "kind": "water",
                "title": { "fr": "Vannes d’arrêt d’eau", "en": "Water shutoff valves" },
                "location": { "fr": "Trappe au plafond des WC", "en": "Hatch in the toilet ceiling" },
                "instruction": {
                    "fr": "Eau froide à gauche, eau chaude à droite. Tournez dans le sens des aiguilles d’une montre.",
                    "en": "Cold water left, hot water right. Turn clockwise."
                }
            },
            {
                "id": "gaz",
                "kind": "gas",
                "title": { "fr": "Robinet de gaz", "en": "Gas valve" },
                "location": { "fr": "Sous l’évier de la cuisine", "en": "Under the kitchen sink" },
                "instruction": {
                    "fr": "Quart de tour : poignée en travers du tuyau = fermé.",
                    "en": "Quarter turn: handle across the pipe means closed."
                }
            },
            {
                "id": "extincteur",
                "kind": "extinguisher",
                "title": { "fr": "Extincteur", "en": "Fire extinguisher" },
                "location": { "fr": "Derrière la porte du cellier", "en": "Behind the pantry door" }
            }
        ],
        "general_note": {
            "fr": "En cas de fuite d’eau, coupez d’abord la vanne, puis appelez-moi à toute heure.",
            "en": "If water leaks, shut the valve first, then call me at any hour."
        }
    })
}

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let context = previews::guest(root).with_config(&sample_config());
    let detail = context.clone().run(render_explore_detail).expect("detail");
    // La carte d'accueil n'a pas de chemin : hors de `previews.json`, mais c'est elle
    // que la démo du livret montre en premier.
    let home = context.run(render_home_card).expect("home");
    previews::check_all(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("explore.detail", detail)],
        vec![("home.card", home)],
    );
}
