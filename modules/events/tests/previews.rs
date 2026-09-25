//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use events::{render_explore_detail, render_upcoming_card};
use serde_json::json;

/// Trois rendez-vous saisis par l'hôte pendant le séjour d'exemple. L'agenda OpenAgenda reste
/// éteint : il demande un appel réseau, et l'aperçu montre ce que l'hôte maîtrise.
fn sample_config() -> serde_json::Value {
    json!({
        "events": [
            {
                "id": "marche-nocturne",
                "title": { "fr": "Marché nocturne des artisans", "en": "Craft night market" },
                "place": { "fr": "Place du village", "en": "Village square" },
                "starts_at": "2026-06-02T18:00:00",
                "ends_at": "2026-06-02T22:00:00",
                "note": { "fr": "Entrée libre", "en": "Free entry" },
                "lat": 43.5528, "lng": 7.0171
            },
            {
                "id": "concert",
                "title": { "fr": "Concert de jazz en plein air", "en": "Open-air jazz concert" },
                "place": { "fr": "Jardin public", "en": "Public garden" },
                "starts_at": "2026-06-04T20:30:00",
                "url": "https://example.com/concert"
            },
            {
                "id": "vide-grenier",
                "title": { "fr": "Vide-grenier du dimanche", "en": "Sunday flea market" },
                "place": { "fr": "Parking de la plage", "en": "Beach car park" },
                "starts_at": "2026-06-07T08:00:00"
            }
        ],
        "disclaimer": { "fr": "Programme indicatif, à vérifier auprès des organisateurs.", "en": "Schedule for guidance, check with the organisers." },
        "nearby_enabled": false
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
