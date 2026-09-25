//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use facility_hours::render_explore_detail;
use serde_json::json;

/// Les équipements d'une résidence de vacances et leurs horaires.
fn sample_config() -> serde_json::Value {
    json!({
        "facilities": [
            {
                "id": "piscine",
                "title": { "fr": "Piscine", "en": "Swimming pool" },
                "lines": [
                    { "fr": "Tous les jours, de juin à septembre", "en": "Every day, June to September" },
                    { "fr": "Enfants sous la surveillance d'un adulte", "en": "Children must be supervised by an adult" }
                ],
                "hours": "9 h – 20 h",
                "note": { "fr": "Bonnet de bain non obligatoire.", "en": "Swim cap not required." }
            },
            {
                "id": "laverie",
                "title": { "fr": "Laverie", "en": "Laundry room" },
                "lines": [{ "fr": "Rez-de-chaussée, bâtiment B", "en": "Ground floor, building B" }],
                "hours": "7 h – 22 h"
            },
            {
                "id": "accueil",
                "title": { "fr": "Accueil de la résidence", "en": "Reception" },
                "lines": [{ "fr": "Du lundi au samedi", "en": "Monday to Saturday" }],
                "hours": "8 h 30 – 12 h · 14 h – 18 h"
            }
        ],
        "general_note": { "fr": "Horaires susceptibles de varier les jours fériés.", "en": "Hours may vary on public holidays." }
    })
}

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let detail = previews::guest(root)
        .with_config(&sample_config())
        .run(render_explore_detail)
        .expect("detail");
    previews::check(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("explore.detail", detail)],
    );
}
