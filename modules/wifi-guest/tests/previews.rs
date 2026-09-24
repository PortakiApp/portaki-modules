//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use serde_json::json;
use wifi_guest::render_explore_detail;

/// Un réseau d'exemple, mot de passe visible tout de suite pour montrer ce que lira le voyageur.
fn sample_config() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "ssid": "Maison-Invites",
        "password": "soleil-2026",
        "hint": "Le réseau 5 GHz est plus rapide dans le salon.",
        "connection_steps": "Choisissez « Maison-Invites » dans les réglages Wi-Fi, puis saisissez le mot de passe.",
        "reveal_policy": "always"
    }))
    .expect("config json")
}

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let detail = previews::guest(root)
        .with_kv("config", sample_config())
        .run(render_explore_detail);
    previews::check(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("explore.detail", detail)],
    );
}
