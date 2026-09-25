//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use access_guide::{render_explore_detail, render_upcoming_card};
use serde_json::json;

/// Une boîte à clés, un digicode d'immeuble et deux étapes d'arrivée, codes visibles tout de
/// suite pour que l'aperçu montre ce que le voyageur lira le jour J.
fn sample_config() -> serde_json::Value {
    json!({
        "primary_method": "keybox",
        "keybox_location": "À droite de la porte d'entrée, sous la boîte aux lettres",
        "keybox_code": "4821",
        "building_access_enabled": true,
        "building_access_gate_code": "A17B",
        "building_access_intercom": "Appartement 3, 2e étage",
        "parking_enabled": true,
        "parking_map_url": "https://maps.example.com/parking",
        "address": "12 rue des Oliviers, 06400 Cannes",
        "reveal_policy": "always",
        "steps": [
            { "id": "parking", "kind": "parking", "title": { "fr": "Garez-vous au sous-sol" },
              "detail": { "fr": "Rampe à gauche de l'immeuble, place n° 8." } },
            { "id": "door", "kind": "door", "title": { "fr": "Entrez dans l'immeuble" },
              "detail": { "fr": "Digicode au portail, puis ascenseur jusqu'au 2e étage." } }
        ],
        "method_instructions": { "fr": "Composez le code, tirez le volet vers le bas : les clés sont sur le crochet." },
        "parking_info": { "fr": "Place n° 8 au sous-sol, badge sur le trousseau." },
        "global_note": { "fr": "Merci de refermer la boîte à clés après usage." }
    })
}

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let context = previews::guest(root).with_config(&sample_config());
    let detail = context
        .clone()
        .run(|ctx| render_explore_detail(ctx).expect("detail"));
    let upcoming = context.run(|ctx| render_upcoming_card(ctx).expect("upcoming"));
    previews::check(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("explore.detail", detail), ("upcoming.card", upcoming)],
    );
}
