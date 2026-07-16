//! Integration-style unit tests with `portaki-test-utils`.

use serial_test::serial;

use emergency_contacts::{
    get_config, render_explore_detail, render_home_card, update_config, UpdateConfigArgs,
};
use portaki_sdk::sdui::component::Component;
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::MockContext;
use serde_json::json;

fn sample_config_bytes() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "contacts_json": r#"[{"id":"samu","label":{"fr":"SAMU","en":"SAMU"},"phone":"15"},{"id":"pompiers","label":{"fr":"Pompiers","en":"Fire"},"phone":"18"}]"#,
        "host_visible_phone": "+33 6 12 34 56 78"
    }))
    .expect("config json")
}

fn contains_component_type(surface: &Surface, type_name: &str) -> bool {
    fn walk(node: &Component, type_name: &str) -> bool {
        let matches = match node {
            Component::Card(_) if type_name == "Card" => true,
            Component::ListItem(_) if type_name == "ListItem" => true,
            Component::Pressable(_) if type_name == "Pressable" => true,
            Component::InfoBanner(_) if type_name == "InfoBanner" => true,
            Component::EmptyState(_) if type_name == "EmptyState" => true,
            Component::Stack(_) if type_name == "Stack" => true,
            Component::Link(_) if type_name == "Link" => true,
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
fn home_card_renders_empty_without_config() {
    MockContext::guest()
        .with_capabilities(&["core.storage"])
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "EmptyState"));
        });
}

#[test]
#[serial]
fn home_card_renders_contacts() {
    MockContext::guest()
        .with_capabilities(&["core.storage"])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "Pressable"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("bottomSheet"));
        });
}

#[test]
#[serial]
fn detail_includes_emergency_banner() {
    MockContext::guest()
        .with_capabilities(&["core.storage"])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_explore_detail(ctx);
            assert!(contains_component_type(&surface, "InfoBanner"));
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
                    contacts: Vec::new(),
                    contacts_json: String::new(),
                    host_visible_phone: "+331234".into(),
                },
            )
            .expect("updateConfig");
            let config = get_config(ctx).expect("getConfig");
            assert_eq!(config.host_visible_phone, "+331234");
        });
}
