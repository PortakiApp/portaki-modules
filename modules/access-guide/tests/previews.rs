//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use access_guide::{
    render_explore_detail, render_home_card, render_status_cell, render_upcoming_card,
};
use serde_json::json;

/// Une maison derrière un portail : le portail, l'allée où se garer, la boîte à clés près de la
/// porte. Codes visibles tout de suite pour que l'aperçu montre ce que le voyageur lira le jour J.
fn sample_config() -> serde_json::Value {
    json!({
        "primary_method": "keybox",
        "keybox_location": "Sur le mur de pierre, à droite de la porte d'entrée",
        "keybox_code": "4821",
        "parking_enabled": true,
        "parking_type": "private",
        "parking_spot": "Allée devant la maison",
        "parking_map_url": "https://maps.example.com/parking",
        "address": "Chemin de la Garoupe, 06160 Antibes",
        "reveal_policy": "always",
        "steps": [
            { "id": "gate", "kind": "gate", "title": { "fr": "Ouvrez le portail" },
              "detail": { "fr": "Portail en bois vert au bout du chemin ; il s'ouvre à la main. Refermez-le derrière vous." } },
            { "id": "parking", "kind": "parking", "title": { "fr": "Garez-vous dans l'allée" },
              "detail": { "fr": "Deux voitures tiennent sur le gravier, devant l'ancienne bergerie." } },
            { "id": "door", "kind": "door", "title": { "fr": "Entrez par la porte d'entrée" },
              "detail": { "fr": "Les clés sont dans la boîte, sur le mur à droite de la porte." } }
        ],
        "method_instructions": { "fr": "Composez le code, tirez le volet vers le bas : les clés sont sur le crochet." },
        "parking_info": { "fr": "Allée privée derrière le portail, gratuite, deux places." },
        "global_note": { "fr": "Merci de refermer la boîte à clés et le portail après usage." }
    })
}

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let context = previews::guest(root).with_config(&sample_config());
    let detail = context
        .clone()
        .run(|ctx| render_explore_detail(ctx).expect("detail"));
    let upcoming = context
        .clone()
        .run(|ctx| render_upcoming_card(ctx).expect("upcoming"));
    let status = context
        .clone()
        .run(|ctx| render_status_cell(ctx).expect("status cell"));
    // La carte d'accueil n'a pas de chemin : hors de `previews.json`, mais c'est elle
    // que la démo du livret montre en premier.
    let home = context.run(render_home_card).expect("home");
    previews::check_all(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![
            ("explore.detail", detail),
            ("status.cell", status),
            ("upcoming.card", upcoming),
        ],
        vec![("home.card", home)],
        None,
    );
}
