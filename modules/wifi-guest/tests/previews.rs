//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use serde_json::json;
use wifi_guest::{render_explore_detail, render_home_card};

/// Un réseau d'exemple, mot de passe visible tout de suite pour montrer ce que lira le voyageur.
fn sample_config() -> serde_json::Value {
    json!({
        "ssid": "Maison-Invites",
        "password": "soleil-2026",
        "hint": "Le réseau 5 GHz est plus rapide dans le salon.",
        "connection_steps": "Choisissez « Maison-Invites » dans les réglages Wi-Fi, puis saisissez le mot de passe.",
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
