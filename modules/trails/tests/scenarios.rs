//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use portaki_test_utils::MockContextBuilder;
use serde_json::json;
use trails as _;

/// Les randonnées des aperçus du catalogue.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    builder.with_config(&json!({
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
                            "elevation_m": 80,
                            "description": {
                                    "fr": "Montée courte par le chemin du Calvaire jusqu’au phare et à sa table d’orientation. Vue sur la baie des Anges.",
                                    "en": "A short climb up the Calvaire path to the lighthouse and its orientation table. View over the Baie des Anges."
                            },
                            "address": "Chemin du Calvaire, Antibes",
                            "lat": 43.5612,
                            "lng": 7.1298,
                            "link_url": "https://www.visorando.com/randonnee-phare-de-la-garoupe"
                    },
                    {
                            "id": "littoral",
                            "level": "easy",
                            "shape": "loop",
                            "title": {
                                    "fr": "Sentier du littoral du Cap",
                                    "en": "Cap coastal path"
                            },
                            "duration_min": 150,
                            "distance_km": 8,
                            "elevation_m": 60,
                            "description": {
                                    "fr": "Le tour du Cap au ras des rochers, entre criques et villas. Passages sur des marches taillées, glissants par mer agitée.",
                                    "en": "Around the Cap at water level, between coves and villas. Carved steps, slippery in rough seas."
                            },
                            "address": "Plage de la Garoupe, Antibes",
                            "lat": 43.5585,
                            "lng": 7.1322
                    },
                    {
                            "id": "brague",
                            "level": "moderate",
                            "shape": "round_trip",
                            "title": {
                                    "fr": "Vallée de la Brague",
                                    "en": "Brague valley"
                            },
                            "duration_min": 180,
                            "distance_km": 9.5,
                            "elevation_m": 220,
                            "description": {
                                    "fr": "Le long de la rivière, sous les chênes, avec plusieurs gués. Agréable par forte chaleur.",
                                    "en": "Along the river, under the oaks, with several fords. Pleasant in hot weather."
                            },
                            "address": "Biot, chemin des Combes",
                            "lat": 43.628,
                            "lng": 7.098
                    },
                    {
                            "id": "baou",
                            "level": "hard",
                            "shape": "round_trip",
                            "title": {
                                    "fr": "Baou de Saint-Jeannet",
                                    "en": "Baou de Saint-Jeannet"
                            },
                            "duration_min": 240,
                            "distance_km": 8,
                            "elevation_m": 600,
                            "description": {
                                    "fr": "Montée raide jusqu’au sommet du baou, panorama de l’Estérel au cap Ferrat. Passages rocheux, chaussures montantes conseillées.",
                                    "en": "A steep climb to the top of the baou, panorama from the Estérel to Cap Ferrat. Rocky sections, ankle boots advised."
                            },
                            "address": "Saint-Jeannet, place Sainte-Barbe",
                            "lat": 43.757,
                            "lng": 7.148,
                            "link_url": "https://www.visorando.com/randonnee-baou-de-saint-jeannet"
                    }
            ],
            "commune_url": "https://www.antibesjuanlespins.com/randonnees"
    }))
}

#[test]
fn every_surface_holds_on_every_case() {
    scenarios::check_surfaces(env!("CARGO_MANIFEST_DIR"), setup);
}

#[test]
fn every_example_runs() {
    scenarios::check_examples(concat!(env!("OUT_DIR"), "/portaki-emissions"), setup, &[]);
}
