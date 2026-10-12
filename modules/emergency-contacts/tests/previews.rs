//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use emergency_contacts::{render_explore_detail, render_home_card};

use serde_json::json;

/// Des contacts, une pharmacie, un médecin et un hôte fictifs, aux numéros des plages réservées à la fiction.
fn sample_config() -> serde_json::Value {
    json!({
        "contacts": [
            { "id": "paulette", "label": { "fr": "Paulette, la voisine", "en": "Paulette, the neighbour" }, "phone": "+33639980012", "note": { "fr": "Elle a un double des clés.", "en": "She has a spare key." } },
            { "id": "plombier", "label": { "fr": "Plombier", "en": "Plumber" }, "phone": "+33199001234", "note": { "fr": "Pour une fuite d'eau", "en": "For a water leak" } }
        ],
        "host_visible_phone": "+33639981234",
        "host_availability": "hours",
        "host_hours": "08:00-21:00",
        "pharmacy": "Pharmacie du Port",
        "pharmacy_phone": "+33199005678",
        "pharmacy_hours": "mon=09:00-19:30;tue=09:00-19:30;wed=09:00-19:30;thu=09:00-19:30;fri=09:00-19:30;sat=09:00-19:30",
        "doctor": "Dr Le Gall",
        "doctor_phone": "+33199009012"
    })
}

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let context = previews::guest(root).with_config(&sample_config());
    let detail = context.clone().run(render_explore_detail).expect("detail");
    // La carte d'accueil n'a pas de chemin : hors de `previews.json`, mais c'est elle
    // que la démo du livret montre en premier.
    let home = context.run(render_home_card).expect("home");
    previews::check_all(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("explore.detail", detail)],
        vec![("home.card", home)],
        None,
    );
}
