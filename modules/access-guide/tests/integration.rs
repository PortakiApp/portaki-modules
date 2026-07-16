//! Integration-style unit tests with `portaki-test-utils`.

use serial_test::serial;

use access_guide::{
    get_config, render_explore_detail, render_home_card, update_config, UpdateConfigArgs,
};
use portaki_sdk::sdui::component::Component;
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::MockContext;
use serde_json::json;

fn sample_config_bytes() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "address": "Ch. des Douaniers",
        "gate_code": "A17B",
        "keybox_code": "4821",
        "parking_info": "Résident · rue Aubernon",
        "parking_map_url": "https://maps.example.com",
        "arrival_video_url": "https://video.example.com",
        "global_note": "Sonnette à gauche",
        "steps_json": r#"[{"id":"1","kind":"parking","title":{"fr":"Se garer","en":"Park"},"detail":{"fr":"Place résident","en":"Resident spot"}}]"#
    }))
    .expect("config json")
}

fn contains_component_type(surface: &Surface, type_name: &str) -> bool {
    fn walk(node: &Component, type_name: &str) -> bool {
        let matches = match node {
            Component::Card(_) if type_name == "Card" => true,
            Component::KeyValue(_) if type_name == "KeyValue" => true,
            Component::Button(_) if type_name == "Button" => true,
            Component::ListItem(_) if type_name == "ListItem" => true,
            Component::Badge(_) if type_name == "Badge" => true,
            Component::InfoBanner(_) if type_name == "InfoBanner" => true,
            Component::EmptyState(_) if type_name == "EmptyState" => true,
            Component::Link(_) if type_name == "Link" => true,
            Component::Stack(_) if type_name == "Stack" => true,
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
        Component::EmptyState(inner) => inner.children.iter().collect(),
        _ => Vec::new(),
    }
}

#[test]
#[serial]
fn home_card_empty_without_config() {
    MockContext::guest()
        .with_capabilities(&["core.storage"])
        .run(|ctx| {
            assert!(contains_component_type(
                &render_home_card(ctx),
                "EmptyState"
            ));
        });
}

#[test]
#[serial]
fn home_card_key_values_and_page() {
    MockContext::guest()
        .with_capabilities(&["core.storage"])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "KeyValue"));
            assert!(contains_component_type(&surface, "Button"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("page"));
        });
}

#[test]
#[serial]
fn detail_has_steps_and_video() {
    MockContext::guest()
        .with_capabilities(&["core.storage"])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_explore_detail(ctx);
            assert!(contains_component_type(&surface, "ListItem"));
            assert!(contains_component_type(&surface, "Badge"));
            assert!(contains_component_type(&surface, "Link"));
        });
}

#[test]
#[serial]
fn update_config_roundtrip() {
    MockContext::host()
        .with_capabilities(&["core.storage"])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    steps: Vec::new(),
                    steps_json: String::new(),
                    parking_map_url: "".into(),
                    arrival_video_url: "".into(),
                    global_note: "".into(),
                    address: "Rue X".into(),
                    gate_code: "1".into(),
                    keybox_code: "".into(),
                    parking_info: "".into(),
                },
            )
            .expect("ok");
            let cfg = get_config(ctx).expect("cfg");
            assert_eq!(cfg.address, "Rue X");
            assert_eq!(cfg.gate_code, "1");
        });
}
