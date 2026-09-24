//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use rules::{render_explore_detail, reset_test_store, save_content, SaveContentArgs};
use serde_json::json;

/// Le règlement type d'une location de vacances.
fn sample_payload() -> String {
    json!({
        "items": [
            { "icon": "clock-circle", "title": "Calme après 22 h", "subtitle": "Merci de penser au voisinage" },
            { "icon": "x", "title": "Logement non-fumeur", "subtitle": "Vous pouvez fumer sur la terrasse" },
            { "icon": "users", "title": "Pas de fête ni d'événement", "subtitle": "" },
            { "icon": "check-circle", "title": "Animaux bienvenus", "subtitle": "Prévenez-nous avant votre arrivée" },
            { "icon": "clock-circle", "title": "Départ avant 10 h", "subtitle": "Laissez les clés sur la table de la cuisine" }
        ]
    })
    .to_string()
}

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    reset_test_store();
    let detail = previews::guest(root).run(|ctx| {
        save_content(
            ctx.clone(),
            SaveContentArgs {
                items: Vec::new(),
                content_fr: sample_payload(),
                content_en: String::new(),
            },
        )
        .expect("save");
        render_explore_detail(ctx)
    });
    previews::check(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("explore.detail", detail)],
    );
}
