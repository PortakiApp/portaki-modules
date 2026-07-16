//! Integration-style unit tests with `portaki-test-utils`.

use serial_test::serial;

use facility_hours::{
    get_config, render_explore_detail, render_home_card, update_config, UpdateConfigArgs,
};
use portaki_sdk::sdui::component::Component;
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::MockContext;
use serde_json::json;

fn sample_config_bytes() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "facilities_json": r#"[{"id":"checkin","title":{"fr":"Arrivée","en":"Check-in"},"hours":"à partir de 16:00","lines":[]},{"id":"pool","title":{"fr":"Piscine","en":"Pool"},"hours":"08:00 – 20:00","lines":[{"fr":"Maillot obligatoire","en":"Swimwear required"}]}]"#,
        "general_note": "Horaires indicatifs"
    }))
    .expect("config json")
}

fn contains_component_type(surface: &Surface, type_name: &str) -> bool {
    fn walk(node: &Component, type_name: &str) -> bool {
        let matches = match node {
            Component::Card(_) if type_name == "Card" => true,
            Component::KeyValue(_) if type_name == "KeyValue" => true,
            Component::ListItem(_) if type_name == "ListItem" => true,
            Component::InfoBanner(_) if type_name == "InfoBanner" => true,
            Component::EmptyState(_) if type_name == "EmptyState" => true,
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
fn home_card_uses_key_value_and_page_overlay() {
    MockContext::guest()
        .with_capabilities(&["core.storage"])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "KeyValue"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("page"));
        });
}

#[test]
#[serial]
fn detail_enriched_list() {
    MockContext::guest()
        .with_capabilities(&["core.storage"])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_explore_detail(ctx);
            assert!(contains_component_type(&surface, "ListItem"));
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
                    facilities: Vec::new(),
                    facilities_json: String::new(),
                    general_note: "note".into(),
                },
            )
            .expect("ok");
            assert_eq!(get_config(ctx).expect("cfg").general_note, "note");
        });
}
