//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use facility_hours::{render_explore_detail, render_home_card};
use serde_json::json;

/// Les équipements d'une résidence de vacances et leurs horaires.
fn sample_config() -> serde_json::Value {
    json!({
        "facilities": [
            {
                "id": "piscine",
                "title": { "fr": "Piscine", "en": "Swimming pool" },
                "group": "Équipements",
                "icon": "sun",
                "opens_at": "09:00",
                "closes_at": "20:00",
                "lines": {
                    "fr": "Tous les jours, de juin à septembre\nEnfants sous la surveillance d'un adulte",
                    "en": "Every day, June to September\nChildren must be supervised by an adult"
                },
                "hours": "9 h – 20 h",
                "note": { "fr": "Bonnet de bain non obligatoire.", "en": "Swim cap not required." }
            },
            {
                "id": "laverie",
                "title": { "fr": "Laverie", "en": "Laundry room" },
                "group": "Équipements",
                "icon": "droplet",
                "opens_at": "07:00",
                "closes_at": "22:00",
                "lines": { "fr": "Rez-de-chaussée, bâtiment B", "en": "Ground floor, building B" },
                "hours": "7 h – 22 h"
            },
            {
                "id": "accueil",
                "title": { "fr": "Accueil de la résidence", "en": "Reception" },
                "group": "Services",
                "icon": "building",
                "opens_at": "08:30",
                "closes_at": "18:00",
                "lines": { "fr": "Du lundi au samedi", "en": "Monday to Saturday" },
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
    );
}
