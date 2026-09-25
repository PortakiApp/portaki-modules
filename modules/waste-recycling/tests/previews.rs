//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use serde_json::json;
use waste_recycling::render_explore_detail;

/// Les bacs d'une commune française type et leurs jours de collecte.
fn sample_config() -> serde_json::Value {
    json!({
        "bins": [
            {
                "id": "jaune",
                "title": { "fr": "Bac jaune", "en": "Yellow bin" },
                "items": [
                    { "fr": "Emballages plastique et métal", "en": "Plastic and metal packaging" },
                    { "fr": "Cartons et papiers", "en": "Cardboard and paper" }
                ],
                "color": "yellow"
            },
            {
                "id": "verre",
                "title": { "fr": "Colonne à verre", "en": "Glass bank" },
                "items": [{ "fr": "Bouteilles et bocaux, sans bouchon", "en": "Bottles and jars, lids off" }],
                "color": "green"
            },
            {
                "id": "biodechets",
                "title": { "fr": "Bac à biodéchets", "en": "Food waste bin" },
                "items": [{ "fr": "Épluchures, marc de café, restes de repas", "en": "Peelings, coffee grounds, leftovers" }],
                "color": "brown"
            },
            {
                "id": "ordures",
                "title": { "fr": "Ordures ménagères", "en": "General waste" },
                "items": [{ "fr": "Tout le reste, en sac fermé", "en": "Everything else, in a closed bag" }],
                "color": "grey"
            }
        ],
        "collection_schedule": {
            "fr": "Bacs à sortir la veille au soir : mardi pour le jaune, vendredi pour les ordures ménagères.",
            "en": "Put the bins out the evening before: Tuesday for yellow, Friday for general waste."
        }
    })
}

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let detail = previews::guest(root)
        .with_config(&sample_config())
        .run(render_explore_detail);
    previews::check(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("explore.detail", detail)],
    );
}
