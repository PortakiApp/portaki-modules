//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use rules::{
    render_explore_detail, render_home_card, reset_test_store, save_content, SaveContentArgs,
};
use serde_json::json;

/// Le règlement type d'une location de vacances — avec statuts et thèmes, pour que l'aperçu du
/// catalogue montre le groupement et les étiquettes et pas seulement une liste plate.
fn sample_payload() -> String {
    json!({
        "items": [
            { "icon": "clock-circle", "title": "Calme après 22 h", "subtitle": "Merci de penser au voisinage", "status": "important", "theme": "Voisinage" },
            { "icon": "x", "title": "Logement non-fumeur", "subtitle": "Vous pouvez fumer sur la terrasse", "status": "important", "theme": "Logement" },
            { "icon": "users", "title": "Pas de fête ni d'événement", "subtitle": "", "status": "important", "theme": "Logement" },
            { "icon": "check-circle", "title": "Animaux bienvenus", "subtitle": "Prévenez-nous avant votre arrivée", "status": "allowed", "theme": "Animaux" },
            { "icon": "clock-circle", "title": "Départ avant 10 h", "subtitle": "Laissez les clés sur la table de la cuisine", "theme": "Départ" }
        ]
    })
    .to_string()
}

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    reset_test_store();
    let (detail, home) = previews::guest(root).run(|ctx| {
        save_content(
            ctx.clone(),
            SaveContentArgs {
                items: Vec::new(),
                content_fr: sample_payload(),
                content_en: String::new(),
            },
        )
        .expect("save");
        (
            render_explore_detail(ctx.clone()).expect("render"),
            render_home_card(ctx).expect("render"),
        )
    });
    previews::check_all(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("explore.detail", detail)],
        vec![("home.card", home)],
        None,
    );
}
