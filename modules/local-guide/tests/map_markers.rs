//! Tests de l'opération `mapMarkers` — ce que le module rend à la carte de la plateforme.

use portaki_sdk::capability;
use serial_test::serial;

use local_guide::{map_markers, MAX_MARKERS};
use portaki_test_utils::MockContext;
use serde_json::json;

fn config_bytes(value: serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(&value).expect("config json")
}

#[test]
#[serial]
fn only_located_spots_become_markers() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv(
            "config",
            config_bytes(json!({
                "spots": [
                    {
                        "id": "plage", "title": { "fr": "Plage du Midi" },
                        "category": "Plage", "lat": 43.548, "lng": 7.005
                    },
                    // Saisi avant la carte : il reste dans la liste, pas sur le plan.
                    { "id": "boulangerie", "title": { "fr": "Boulangerie" } }
                ]
            })),
        )
        .run(|ctx| {
            let response = map_markers(ctx).expect("markers");
            assert_eq!(response.markers.len(), 1);
            let marker = &response.markers[0];
            assert_eq!(marker.id, "plage");
            assert_eq!(marker.label.as_deref(), Some("Plage du Midi"));
            assert_eq!(marker.category.as_deref(), Some("Plage"));
            assert!((marker.lat - 43.548).abs() < 1e-9);
            assert!((marker.lng - 7.005).abs() < 1e-9);
        });
}

#[test]
#[serial]
fn null_island_is_not_a_marker() {
    // Deux champs de position laissés vides par un formulaire : ce n'est pas un lieu.
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv(
            "config",
            config_bytes(json!({
                "spots": [{ "id": "s1", "title": { "fr": "Plage" }, "lat": 0.0, "lng": 0.0 }]
            })),
        )
        .run(|ctx| {
            assert!(map_markers(ctx).expect("markers").markers.is_empty());
        });
}

#[test]
#[serial]
fn a_module_without_config_answers_an_empty_list() {
    // Le module n'a rien à dire, ce qui n'est pas une erreur : la carte se passe de lui.
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            assert!(map_markers(ctx).expect("markers").markers.is_empty());
        });
}

#[test]
#[serial]
fn the_label_follows_the_guest_locale() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv(
            "config",
            config_bytes(json!({
                "spots": [{
                    "id": "s1",
                    "title": { "fr": "Marché couvert", "en": "Covered market" },
                    "lat": 43.55, "lng": 7.01
                }]
            })),
        )
        .run(|ctx| {
            let expected = if ctx.locale.starts_with("en") {
                "Covered market"
            } else {
                "Marché couvert"
            };
            let response = map_markers(ctx).expect("markers");
            assert_eq!(response.markers[0].label.as_deref(), Some(expected));
        });
}

#[test]
#[serial]
fn the_cap_stops_a_long_list() {
    let spots: Vec<serde_json::Value> = (0..MAX_MARKERS + 12)
        .map(|index| {
            json!({
                "id": format!("spot-{index}"),
                "title": { "fr": format!("Lieu {index}") },
                "lat": 43.55, "lng": 7.01
            })
        })
        .collect();
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", config_bytes(json!({ "spots": spots })))
        .run(|ctx| {
            let response = map_markers(ctx).expect("markers");
            assert_eq!(response.markers.len(), MAX_MARKERS);
            // La coupe garde le début de la liste de l'hôte, elle ne réordonne rien.
            assert_eq!(response.markers[0].id, "spot-0");
        });
}
