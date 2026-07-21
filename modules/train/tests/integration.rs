//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::sdui::component::Component;
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::{MockContext, Property};
use serde_json::json;

use train::{render_explore_detail, render_home_card};

fn contains_component_type(surface: &Surface, type_name: &str) -> bool {
    fn walk(node: &Component, type_name: &str) -> bool {
        let matches = match node {
            Component::Card(_) if type_name == "Card" => true,
            Component::TimedEntry(_) if type_name == "TimedEntry" => true,
            Component::FilterChip(_) if type_name == "FilterChip" => true,
            Component::FilterBar(_) if type_name == "FilterBar" => true,
            Component::KeyValue(_) if type_name == "KeyValue" => true,
            Component::Stack(_) if type_name == "Stack" => true,
            Component::Text(_) if type_name == "Text" => true,
            Component::EmptyState(_) if type_name == "EmptyState" => true,
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
        Component::FilterBar(inner) => inner.children.iter().collect(),
        Component::EmptyState(inner) => inner.children.iter().collect(),
        _ => Vec::new(),
    }
}

#[test]
fn home_card_shows_board_glance() {
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let card = render_home_card(ctx);
            assert!(contains_component_type(&card, "Card"));
            assert!(contains_component_type(&card, "TimedEntry"));

            let card_json = serde_json::to_string(&card).expect("json");
            assert!(card_json.contains("\"type\":\"openOverlay\""));
            assert!(card_json.contains("explore.detail"));
            assert!(card_json.contains("Nice-Ville"));
        });
}

#[test]
fn explore_detail_defaults_to_nice_ville_and_lists_filter_chips() {
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let detail = render_explore_detail(ctx);
            assert!(contains_component_type(&detail, "FilterChip"));
            assert!(contains_component_type(&detail, "TimedEntry"));
            assert!(contains_component_type(&detail, "KeyValue"));

            let detail_json = serde_json::to_string(&detail).expect("json");
            assert!(detail_json.contains("Nice-Ville"));
            assert!(detail_json.contains("Cannes"));
            assert!(detail_json.contains("Monaco"));
            assert!(detail_json.contains("Grasse"));
            assert!(detail_json.contains("\"selected\":true"));
        });
}

#[test]
fn explore_detail_honors_dest_param() {
    MockContext::guest()
        .with_property(Property::default())
        .run(|mut ctx| {
            ctx.input = json!({ "dest": "Cannes" });
            let detail = render_explore_detail(ctx);
            let detail_json = serde_json::to_string(&detail).expect("json");
            assert!(detail_json.contains("\"value\":\"Cannes\""));
            assert!(detail_json.contains("quai 3"));
        });
}

#[test]
fn explore_detail_falls_back_to_default_on_unknown_dest() {
    MockContext::guest()
        .with_property(Property::default())
        .run(|mut ctx| {
            ctx.input = json!({ "dest": "Marseille" });
            let detail = render_explore_detail(ctx);
            let detail_json = serde_json::to_string(&detail).expect("json");
            assert!(detail_json.contains("\"value\":\"Nice-Ville\""));
        });
}
