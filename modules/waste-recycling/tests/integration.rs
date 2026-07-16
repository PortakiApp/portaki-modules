//! Integration-style unit tests with `portaki-test-utils`.

use serial_test::serial;

use portaki_sdk::sdui::component::Component;
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::MockContext;
use serde_json::json;

use waste_recycling::{
    get_config, render_explore_detail, render_home_card, update_config, BinInput, UpdateConfigArgs,
};

fn sample_config_bytes() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "bins": [
            {
                "id": "yellow",
                "title": {"fr": "Bac jaune", "en": "Yellow bin"},
                "items": [{"fr": "Emballages, plastique", "en": "Packaging, plastic"}],
                "color": "#f4c020"
            },
            {
                "id": "green",
                "title": {"fr": "Bac vert", "en": "Green bin"},
                "items": [{"fr": "Verre", "en": "Glass"}],
                "color": "#3a8a4d"
            }
        ],
        "collection_schedule": "Mardi & vendredi matin"
    }))
    .expect("config json")
}

fn contains_component_type(surface: &Surface, type_name: &str) -> bool {
    fn walk(node: &Component, type_name: &str) -> bool {
        let matches = match node {
            Component::Card(_) if type_name == "Card" => true,
            Component::ListItem(_) if type_name == "ListItem" => true,
            Component::ColorDotItem(_) if type_name == "ColorDotItem" => true,
            Component::InfoBanner(_) if type_name == "InfoBanner" => true,
            Component::EmptyState(_) if type_name == "EmptyState" => true,
            Component::Stack(_) if type_name == "Stack" => true,
            Component::Form(_) if type_name == "Form" => true,
            Component::Page(_) if type_name == "Page" => true,
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
        Component::Form(inner) => inner.children.iter().collect(),
        Component::Page(inner) => inner.children.iter().collect(),
        Component::Field(inner) => inner.children.iter().collect(),
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
fn home_card_renders_bins_with_config() {
    MockContext::guest()
        .with_capabilities(&["core.storage"])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "ColorDotItem"));
            assert!(contains_component_type(&surface, "InfoBanner"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("bottomSheet"));
            assert!(json.contains("explore.detail"));
        });
}

#[test]
#[serial]
fn detail_renders_enriched_bins() {
    MockContext::guest()
        .with_capabilities(&["core.storage"])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_explore_detail(ctx);
            assert!(contains_component_type(&surface, "Stack"));
            assert!(contains_component_type(&surface, "ColorDotItem"));
        });
}

#[test]
#[serial]
fn update_config_persists_and_get_config_reads() {
    MockContext::host()
        .with_capabilities(&["core.storage"])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    bins: vec![BinInput {
                        title_fr: "A".into(),
                        title_en: "A".into(),
                        items_fr: String::new(),
                        color: String::new(),
                    }],
                    bins_json: String::new(),
                    collection_schedule: "Lundi".into(),
                },
            )
            .expect("updateConfig");
            let config = get_config(ctx).expect("getConfig");
            assert_eq!(config.bins.len(), 1);
            assert_eq!(config.bins[0].title.fr, "A");
            assert_eq!(config.collection_schedule, "Lundi");
        });
}
