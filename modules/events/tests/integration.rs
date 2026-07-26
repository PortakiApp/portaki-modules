//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use events::{
    get_config, render_explore_detail, render_home_card, update_config, EventInput,
    UpdateConfigArgs,
};
use portaki_sdk::sdui::component::Component;
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::MockContext;
use serde_json::json;

fn sample_config_bytes() -> Vec<u8> {
    serde_json::to_vec(&json!({
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
    }))
    .expect("config json")
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

fn contains_component_type(surface: &Surface, type_name: &str) -> bool {
    fn walk(node: &Component, type_name: &str) -> bool {
        let matches = match node {
            Component::Card(_) if type_name == "Card" => true,
            Component::ListItem(_) if type_name == "ListItem" => true,
            Component::Pill(_) if type_name == "Pill" => true,
            Component::Pressable(_) if type_name == "Pressable" => true,
            Component::InfoBanner(_) if type_name == "InfoBanner" => true,
            Component::EmptyState(_) if type_name == "EmptyState" => true,
            Component::Link(_) if type_name == "Link" => true,
            Component::Stack(_) if type_name == "Stack" => true,
            Component::Map(_) if type_name == "Map" => true,
            _ => false,
        };
        if matches {
            return true;
        }
        for child in child_components(node) {
            if walk(child, type_name) {
                return true;
            }
        }
        false
    }
    walk(&surface.root, type_name)
}

fn child_components(node: &Component) -> Vec<&Component> {
    match node {
        Component::Stack(inner) => inner.children.iter().collect(),
        Component::Card(inner) => inner.children.iter().collect(),
        Component::ListItem(inner) => inner.children.iter().collect(),
        Component::Pressable(inner) => inner.children.iter().collect(),
        Component::EmptyState(inner) => inner.children.iter().collect(),
        _ => Vec::new(),
    }
}

#[test]
#[serial]
fn home_card_empty_without_config() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            assert!(contains_component_type(
                &render_home_card(ctx),
                "EmptyState"
            ));
        });
}

#[test]
#[serial]
fn home_card_renders_events_with_pressable_link() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "ListItem"));
            assert!(contains_component_type(&surface, "Pressable"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("bottomSheet"));
        });
}

#[test]
#[serial]
fn detail_includes_map_and_link() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_explore_detail(ctx);
            assert!(contains_component_type(&surface, "Map"));
            assert!(contains_component_type(&surface, "Link"));
            assert!(contains_component_type(&surface, "InfoBanner"));
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
        .with_kv(
            "config",
            serde_json::to_vec(&json!({
                "nearby_enabled": true,
                "radius_km": 40
            }))
            .unwrap(),
        )
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "ListItem"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("Festival du port") || json.contains("Pressable"));
        });
}

#[test]
#[serial]
fn update_config_roundtrip() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    events: vec![EventInput {
                        title: "Fête du village".into(),
                        place: "Place centrale".into(),
                        starts_at: "2099-08-01T20:00:00Z".into(),
                        url: String::new(),
                        lat: String::new(),
                        lng: String::new(),
                    }],
                    disclaimer: "d".into(),
                    nearby_enabled: "true".into(),
                    radius_km: "20".into(),
                },
            )
            .expect("ok");
            let cfg = get_config(ctx).expect("cfg");
            assert_eq!(cfg.disclaimer.get("fr"), "d");
            assert_eq!(cfg.parse_events().len(), 1);
            assert_eq!(cfg.parse_events()[0].title.get("fr"), "Fête du village");
            assert!(cfg.nearby_enabled);
            assert_eq!(cfg.radius_km, 20);
        });
}
