//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use guest_reviews::render_post_stay_card;
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
    let card = previews::guest(root)
        .with_config(&sample_config())
        .run(render_post_stay_card);
    previews::check(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("post-stay.card", card)],
    );
}
