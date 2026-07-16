//! Integration-style unit tests with `portaki-test-utils`.

use serial_test::serial;

use local_guide::{
    get_config, render_explore_detail, render_home_card, update_config, UpdateConfigArgs,
};
use portaki_sdk::sdui::component::Component;
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::MockContext;
use serde_json::json;

fn sample_config_bytes() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "spots_json": r#"[{"id":"bike","title":{"fr":"Holiday Bikes","en":"Holiday Bikes"},"category":"Location vélos","distance":"900 m","tag":"1j offert","url":"https://example.com","detail":{"fr":"Vélos ville et électriques.","en":"City and e-bikes."}}]"#,
        "disclaimer": "Suggestions non partenaires"
    }))
    .expect("config json")
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
fn home_card_renders_spots_with_pill() {
    MockContext::guest()
        .with_capabilities(&["core.storage"])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "Pill"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("bottomSheet"));
        });
}

#[test]
#[serial]
fn detail_includes_link() {
    MockContext::guest()
        .with_capabilities(&["core.storage"])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_explore_detail(ctx);
            assert!(contains_component_type(&surface, "Link"));
            assert!(contains_component_type(&surface, "InfoBanner"));
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
                    spots: Vec::new(),
                    spots_json: String::new(),
                    disclaimer: "d".into(),
                },
            )
            .expect("ok");
            assert_eq!(get_config(ctx).expect("cfg").disclaimer, "d");
        });
}
