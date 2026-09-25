//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use events::{render_explore_detail, render_home_card, render_host_main, render_upcoming_card};
use portaki_test_utils::{MockContext, SurfaceAssertions};
use serde_json::json;

#[path = "../../../support/config_form.rs"]
mod config_form;

fn sample_config() -> serde_json::Value {
    json!({
        "events": [{
            "id": "evt-1",
            "title": {"fr": "Concert jazz", "en": "Jazz concert"},
            "place": {"fr": "Théâtre de la Mer", "en": "Sea theatre"},
            "starts_at": "2099-07-25T18:00:00Z",
            "url": "https://example.com/tickets",
            "lat": 43.58,
            "lng": 7.12,
            "note": {"fr": "Arrivez tôt.", "en": "Arrive early."}
        }],
        "disclaimer": "Dates indicatives",
        "nearby_enabled": false
    })
}

fn openagenda_payload() -> String {
    json!({
        "total": 1,
        "events": [{
            "uid": 56158955,
            "title": "Festival du port",
            "location": {
                "name": "Quai",
                "city": "Cannes",
                "latitude": 43.55,
                "longitude": 7.01
            },
            "nextTiming": { "begin": "2099-07-28T19:00:00.000Z" },
            "canonicalUrl": "https://openagenda.com/events/festival-du-port"
        }]
    })
    .to_string()
}

#[test]
#[serial]
fn home_card_empty_without_config() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            assert!(
                SurfaceAssertions::new(&render_home_card(ctx).expect("surface"))
                    .contains_type("EmptyState")
            );
        });
}

#[test]
#[serial]
fn home_card_renders_events_with_pressable_link() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Pressable"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("bottomSheet"));
        });
}

#[test]
#[serial]
fn upcoming_card_is_compact_with_next_event_headline() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_upcoming_card(ctx).expect("surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Text"));
            // Compact: no full event list / map on the prep card.
            assert!(!SurfaceAssertions::new(&surface).contains_type("ListItem"));
            assert!(!SurfaceAssertions::new(&surface).contains_type("Map"));

            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("upcoming.card"));
            assert!(json.contains("Concert jazz") || json.contains("Jazz concert"));
        });
}

#[test]
#[serial]
fn upcoming_card_empty_without_config() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            assert!(
                SurfaceAssertions::new(&render_upcoming_card(ctx).expect("surface"))
                    .contains_type("EmptyState")
            );
        });
}

#[test]
#[serial]
fn detail_includes_map_and_link() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Map"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Link"));
            assert!(SurfaceAssertions::new(&surface).contains_type("InfoBanner"));
        });
}

#[test]
#[serial]
fn home_card_renders_openagenda_nearby() {
    MockContext::guest()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::external::OPEN_AGENDA_POOL,
        ])
        .with_connector_response("open-agenda", "nearby_events", openagenda_payload())
        .with_config(&json!({
            "nearby_enabled": true,
            "radius_km": 40
        }))
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("Festival du port") || json.contains("Pressable"));
        });
}

#[test]
#[serial]
fn the_host_form_sends_the_declared_keys() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            config_form::assert_form_matches_config(
                concat!(env!("OUT_DIR"), "/portaki-emissions"),
                &surface,
                &[],
            );
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("Concert jazz"));
            assert!(json.contains("Dates indicatives"));
        });
}

/// A property not geocoded yet: no nearby search, no call, and a guest empty state.
#[test]
#[serial]
fn without_a_position_nothing_is_searched_nearby() {
    MockContext::guest()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::external::OPEN_AGENDA_POOL,
        ])
        .with_coordinates(None)
        .with_connector_response("open-agenda", "nearby_events", openagenda_payload())
        .with_config(&json!({ "nearby_enabled": true }))
        .run_with(|ctx, host| {
            let detail = render_explore_detail(ctx.clone()).expect("surface");
            assert!(SurfaceAssertions::new(&detail).contains_type("EmptyState"));
            let json = serde_json::to_string(&detail).expect("json");
            assert!(json.contains("i18n:guest.empty.title"), "{json}");
            assert!(render_home_card(ctx)
                .map(|s| SurfaceAssertions::new(&s).contains_type("EmptyState"))
                .expect("surface"));
            assert!(host.connector_calls().is_empty());
        });
}
