//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use events::{
    map_markers, render_explore_detail, render_explore_item, render_home_card, render_upcoming_card,
};
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
                "note": { "fr": "Une trentaine d'artisans, de la poterie au miel du pays.", "en": "Thirty-odd makers, from pottery to local honey." },
                "price": "Entrée libre",
                "tips": {
                    "fr": "Venez avant 19 h, la place se remplit vite.\nLe stand de socca est au fond, côté fontaine.",
                    "en": "Come before 7pm, the square fills up fast.\nThe socca stall is at the back, by the fountain."
                },
                "access": {
                    "fr": "Place piétonne, accès de plain-pied\nParking de l'église à 200 m",
                    "en": "Pedestrian square, step-free access\nChurch car park 200 m away"
                },
                "address": "Place du village, Cannes",
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
    let item = context.clone().run(|mut ctx| {
        ctx.input = json!({ "eventId": "marche-nocturne" });
        render_explore_item(ctx).expect("surface")
    });
    let upcoming = context
        .clone()
        .run(|ctx| render_upcoming_card(ctx).expect("surface"));
    // La carte d'accueil n'a pas de chemin : hors de `previews.json`, mais c'est elle
    // que la démo du livret montre en premier.
    // Les points que ce module pose sur la carte du livret (§3). La démo les servait vides.
    let markers = context
        .clone()
        .run(|ctx| map_markers(ctx).expect("repères"));
    let home = context.run(render_home_card).expect("home");
    previews::check_all(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![
            ("explore.detail", detail),
            ("explore.item", item),
            ("upcoming.card", upcoming),
        ],
        vec![("home.card", home)],
        Some(serde_json::to_value(markers).expect("repères")),
    );
}
