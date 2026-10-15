//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use ev_parking::{render_explore_detail, render_home_card};
use serde_json::json;

/// Une borne dans le garage de la maison, codes visibles tout de suite pour montrer ce que lira le voyageur.
fn sample_config() -> serde_json::Value {
    json!({
        "spot_label": "Garage de la maison, à gauche du portail",
        "charger_pin": "2468",
        "map_url": "https://maps.example.com/parking",
        "instructions": "Branchez le câble Type 2 fourni, puis saisissez le PIN sur la borne.",
        "reveal_policy": "always"
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
        None,
    );
}
