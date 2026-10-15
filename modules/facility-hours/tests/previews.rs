//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use facility_hours::{render_explore_detail, render_home_card};
use serde_json::json;

/// Les équipements de la maison et leurs horaires : la piscine, le jacuzzi du jardin (qu'on
/// coupe le soir pour les voisins), le ménage en cours de séjour avec sa pause de midi.
fn sample_config() -> serde_json::Value {
    json!({
        "facilities": [
            {
                "id": "piscine",
                "title": { "fr": "Piscine", "en": "Swimming pool" },
                "group": "Équipements",
                "icon": "droplet",
                "opens_at": "09:00",
                "closes_at": "20:00",
                "lines": {
                    "fr": "Chauffée de mai à octobre\nEnfants sous la surveillance d'un adulte",
                    "en": "Heated from May to October\nChildren must be supervised by an adult"
                },
                "hours": "9 h – 20 h",
                "note": { "fr": "Merci de remettre la bâche le soir.", "en": "Please put the cover back on in the evening." }
            },
            {
                "id": "jacuzzi",
                "title": { "fr": "Jacuzzi", "en": "Hot tub" },
                "group": "Équipements",
                "icon": "sparkles",
                "opens_at": "08:00",
                "closes_at": "22:00",
                "lines": { "fr": "Sous la pergola, côté jardin", "en": "Under the pergola, garden side" },
                "hours": "8 h – 22 h"
            },
            {
                "id": "menage",
                "title": { "fr": "Ménage en cours de séjour", "en": "Mid-stay cleaning" },
                "group": "Services",
                "icon": "sparkles",
                "opens_at": "08:30",
                "closes_at": "18:00",
                "break_from": "12:00",
                "break_to": "14:00",
                "lines": { "fr": "Du lundi au samedi, sur demande la veille", "en": "Monday to Saturday, on request the day before" },
                "hours": "8 h 30 – 12 h · 14 h – 18 h"
            }
        ],
        "general_note": { "fr": "Horaires susceptibles de varier les jours fériés.", "en": "Hours may vary on public holidays." }
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
