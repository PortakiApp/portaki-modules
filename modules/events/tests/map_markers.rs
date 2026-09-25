//! Tests de l'opération `mapMarkers` — ce que le module rend à la carte de la plateforme.

use portaki_sdk::capability;
use serial_test::serial;

use events::map_markers;
use portaki_test_utils::MockContext;
use serde_json::json;

/// `nearby_enabled: false` : ces tests décrivent les événements de l'hôte, sans aller
/// chercher l'agenda alentour — OpenAgenda a ses propres tests.
fn local_events() -> serde_json::Value {
    json!({
        "nearby_enabled": false,
        "events": [
            {
                "id": "evt-1",
                "title": { "fr": "Concert jazz", "en": "Jazz concert" },
                "place": { "fr": "Théâtre de la Mer" },
                "starts_at": "2099-07-25T18:00:00Z",
                "lat": 43.58, "lng": 7.12
            },
            {
                // Sans lieu connu : il reste dans la liste, pas sur la carte.
                "id": "evt-2",
                "title": { "fr": "Brocante" },
                "starts_at": "2099-07-26T09:00:00Z"
            }
        ]
    })
}

#[test]
#[serial]
fn only_located_events_become_markers() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&(local_events()))
        .run(|ctx| {
            let response = map_markers(ctx).expect("markers");
            assert_eq!(response.markers.len(), 1);
            let marker = &response.markers[0];
            assert_eq!(marker.id, "evt-1");
            assert_eq!(marker.label.as_deref(), Some("Concert jazz"));
            assert!((marker.lat - 43.58).abs() < 1e-9);
        });
}

#[test]
#[serial]
fn an_empty_agenda_answers_an_empty_list() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&(json!({ "nearby_enabled": false, "events": [] })))
        .run(|ctx| {
            assert!(map_markers(ctx).expect("markers").markers.is_empty());
        });
}

#[test]
#[serial]
fn null_island_is_not_a_marker() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(
            &(json!({
                "nearby_enabled": false,
                "events": [{
                    "id": "evt-1", "title": { "fr": "Concert" },
                    "starts_at": "2099-07-25T18:00:00Z", "lat": 0.0, "lng": 0.0
                }]
            })),
        )
        .run(|ctx| {
            assert!(map_markers(ctx).expect("markers").markers.is_empty());
        });
}
