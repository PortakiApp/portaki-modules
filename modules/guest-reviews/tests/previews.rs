//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use guest_reviews::{render_home_card, render_post_stay_card};
use serde_json::json;

/// L'avis déposé sur Portaki seulement : un lien de plateforme d'exemple pointerait vers une
/// annonce qui n'existe pas.
fn sample_config() -> serde_json::Value {
    json!({
        "platform_airbnb": false,
        "platform_portaki": true,
        "thank_you_message": {
            "fr": "Merci pour votre séjour ! Votre avis aide les prochains voyageurs à nous choisir.",
            "en": "Thank you for staying with us! Your review helps future guests choose us."
        }
    })
}

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let context = previews::guest(root).with_config(&sample_config());
    let card = context
        .clone()
        .run(render_post_stay_card)
        .expect("post-stay card");
    // La carte d'accueil n'a pas de chemin : hors de `previews.json`, mais c'est elle
    // que la démo du livret montre en premier.
    let home = context.run(render_home_card).expect("home");
    previews::check_all(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("post-stay.card", card)],
        vec![("home.card", home)],
    );
}
