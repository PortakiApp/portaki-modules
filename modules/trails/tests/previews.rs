//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use serde_json::json;
use trails::{map_markers, render_explore_detail, render_explore_item, render_home_card};

/// Trois balades autour du Cap d'Antibes, départs relevés sur OpenStreetMap : deux faciles, une
/// moyenne, avec et sans fiche tierce.
fn sample_config() -> serde_json::Value {
    json!({
            "trails": [
                    {
                            "id": "garoupe",
                            "level": "easy",
                            "shape": "round_trip",
                            "title": {
                                    "fr": "Phare de la Garoupe",
                                    "en": "La Garoupe lighthouse"
                            },
                            "duration_min": 60,
                            "distance_km": 2.6,
                            "elevation_m": 100,
                            "description": {
                                    "fr": "De la plage de la Salis, montée par le chemin du Calvaire jusqu’au phare et à la chapelle de la Garoupe. Vue sur la baie des Anges et les Alpes.",
                                    "en": "From La Salis beach, up the Calvaire path to the lighthouse and the Garoupe chapel. Views over the Baie des Anges and the Alps."
                            },
                            "address": "Plage de la Salis, boulevard James Wyllie, Antibes",
                            "lat": 43.57147,
                            "lng": 7.12761,
                            "link_url": "https://www.openstreetmap.org/way/670533618"
                    },
                    {
                            "id": "littoral",
                            "level": "moderate",
                            "shape": "loop",
                            "title": {
                                    "fr": "Sentier du littoral (Tire-Poil)",
                                    "en": "Coastal path (Tire-Poil)"
                            },
                            "duration_min": 120,
                            "distance_km": 5,
                            "elevation_m": 40,
                            "description": {
                                    "fr": "De la plage de la Garoupe à la villa Eilenroc au ras des rochers, retour par les chemins du Cap. Marches taillées, glissantes par mer agitée.",
                                    "en": "From La Garoupe beach to Villa Eilenroc along the rocks, back through the lanes of the Cap. Carved steps, slippery in rough seas."
                            },
                            "address": "Plage de la Garoupe, Antibes",
                            "lat": 43.55529,
                            "lng": 7.13969
                    },
                    {
                            "id": "remparts",
                            "level": "easy",
                            "shape": "round_trip",
                            "title": {
                                    "fr": "Des remparts au Fort Carré",
                                    "en": "From the ramparts to Fort Carré"
                            },
                            "duration_min": 60,
                            "distance_km": 3,
                            "elevation_m": 30,
                            "description": {
                                    "fr": "Le long des remparts du vieil Antibes, au pied du musée Picasso, puis tout le port Vauban jusqu’au Fort Carré.",
                                    "en": "Along the ramparts of old Antibes, below the Picasso museum, then the length of Port Vauban to Fort Carré."
                            },
                            "address": "Promenade de l’Amiral de Grasse, Antibes",
                            "lat": 43.58006,
                            "lng": 7.12808
                    }
            ],
            "commune_url": "https://www.antibesjuanlespins.com/"
    })
}

#[test]
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let (list, item, home) = previews::guest(root)
        .with_config(&sample_config())
        .run(|ctx| {
            let mut item_ctx = ctx.clone();
            item_ctx.input = json!({ "trailId": "garoupe" });
            (
                render_explore_detail(ctx.clone()).expect("list"),
                render_explore_item(item_ctx).expect("item"),
                render_home_card(ctx).expect("home"),
            )
        });
    // Les points que ce module pose sur la carte du livret (§3). La démo les servait vides.
    let markers = previews::guest(root)
        .with_config(&sample_config())
        .run(|ctx| map_markers(ctx).expect("repères"));
    // Chaque fiche : la démo ouvre celle de l'itinéraire tapé, pas toujours la même.
    let variants = ["garoupe", "littoral", "remparts"]
        .into_iter()
        .map(|id| {
            let input = json!({ "trailId": id });
            let tree = previews::guest(root)
                .with_config(&sample_config())
                .run(|mut ctx| {
                    ctx.input = input.clone();
                    render_explore_item(ctx).expect("item")
                });
            ("explore.item", input, tree)
        })
        .collect();
    previews::check_all_with_variants(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("explore.detail", list), ("explore.item", item)],
        vec![("home.card", home)],
        Some(serde_json::to_value(markers).expect("repères")),
        Vec::new(),
        variants,
    );
}
