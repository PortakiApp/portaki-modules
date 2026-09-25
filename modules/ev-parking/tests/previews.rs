//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use ev_parking::render_explore_detail;
use serde_json::json;

/// Une borne en sous-sol, codes visibles tout de suite pour montrer ce que lira le voyageur.
fn sample_config() -> serde_json::Value {
    json!({
        "spot_label": "Place n° 8, niveau -1",
        "charger_pin": "2468",
        "parking_code": "1357",
        "map_url": "https://maps.example.com/parking",
        "instructions": "Branchez le câble Type 2 fourni, puis saisissez le PIN sur la borne.",
        "reveal_policy": "always"
    })
}

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let detail = previews::guest(root)
        .with_config(&sample_config())
        .run(render_explore_detail)
        .expect("detail");
    previews::check(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("explore.detail", detail)],
    );
}
