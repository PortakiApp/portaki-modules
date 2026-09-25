//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use local_guide::{render_explore_detail, render_upcoming_card};

use serde_json::json;

/// Trois adresses d'exemple autour du logement fictif des fixtures (Cannes). Activités et
/// billets restent éteints, comme chez un hôte qui vient d'installer le module.
fn sample_config() -> serde_json::Value {
    json!({
        "spots": [
            {
                "id": "boulangerie",
                "title": { "fr": "La boulangerie du village", "en": "The village bakery" },
                "category": "Boulangerie",
                "distance": "350 m",
                "detail": { "fr": "Croissants dès 7 h, fermée le lundi.", "en": "Croissants from 7am, closed on Mondays." },
                "lat": 43.5531, "lng": 7.0152
            },
            {
                "id": "marche",
                "title": { "fr": "Marché provençal", "en": "Provençal market" },
                "category": "Marché",
                "distance": "1,2 km",
                "tag": "Samedi matin",
                "detail": { "fr": "Fruits, fromages et fleurs, le samedi matin.", "en": "Fruit, cheese and flowers, on Saturday mornings." },
                "lat": 43.5552, "lng": 7.0214
            },
            {
                "id": "lac",
                "title": { "fr": "Lac de Saint-Cassien", "en": "Lake Saint-Cassien" },
                "category": "Baignade",
                "distance": "9 km",
                "detail": { "fr": "Eau calme et plage de galets, idéal avec des enfants.", "en": "Calm water and a pebble beach, great with kids." },
                "lat": 43.5663, "lng": 6.8511
            }
        ],
        "disclaimer": { "fr": "Suggestions de votre hôte, sans partenariat.", "en": "Your host's picks, no partnership." }
    })
}

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let context = previews::guest(root).with_config(&sample_config());
    let detail = context
        .clone()
        .run(|ctx| render_explore_detail(ctx).expect("surface"));
    let upcoming = context.run(|ctx| render_upcoming_card(ctx).expect("surface"));
    previews::check(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("explore.detail", detail), ("upcoming.card", upcoming)],
    );
}
