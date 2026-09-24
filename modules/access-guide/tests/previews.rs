//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use access_guide::{render_explore_detail, render_upcoming_card};
use serde_json::json;

/// Une boîte à clés, un digicode d'immeuble et deux étapes d'arrivée, codes visibles tout de
/// suite pour que l'aperçu montre ce que le voyageur lira le jour J.
fn sample_config() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "primary_method": "keybox",
        "method": { "kind": "keybox", "location": "À droite de la porte d'entrée, sous la boîte aux lettres", "code": "4821" },
        "building_access": { "gate_code": "A17B", "intercom": "Appartement 3, 2e étage" },
        "parking": { "map_url": "https://maps.example.com/parking" },
        "arrival": {
            "address": "12 rue des Oliviers, 06400 Cannes",
            "steps": [{ "id": "parking", "kind": "parking" }, { "id": "entree", "kind": "door" }]
        },
        "reveal_policy": "always"
    }))
    .expect("config json")
}

fn sample_texts() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "method_instructions": "Composez le code, tirez le volet vers le bas : les clés sont sur le crochet.",
        "parking_info": "Place n° 8 au sous-sol, badge sur le trousseau.",
        "global_note": "Merci de refermer la boîte à clés après usage.",
        "steps": [
            { "id": "parking", "title": "Garez-vous au sous-sol", "detail": "Rampe à gauche de l'immeuble, place n° 8." },
            { "id": "entree", "title": "Entrez dans l'immeuble", "detail": "Digicode au portail, puis ascenseur jusqu'au 2e étage." }
        ]
    }))
    .expect("texts json")
}

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let context = previews::guest(root)
        .with_kv("config", sample_config())
        .with_kv("texts/fr", sample_texts());
    let detail = context.clone().run(render_explore_detail);
    let upcoming = context.run(render_upcoming_card);
    previews::check(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("explore.detail", detail), ("upcoming.card", upcoming)],
    );
}
