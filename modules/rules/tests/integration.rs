//! Integration-style unit tests with `portaki-test-utils`.

use chrono::Utc;
use serial_test::serial;
use uuid::Uuid;

use portaki_sdk::sdui::component::Component;
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::{MockContext, Property};
use serde_json::json;

use rules::{
    get_content, render_explore_detail, render_home_card, reset_test_store, save_content,
    GetContentArgs, RulesContent, SaveContentArgs,
};

fn contains_component_type(surface: &Surface, type_name: &str) -> bool {
    fn walk(node: &Component, type_name: &str) -> bool {
        let matches = match node {
            Component::Card(_) if type_name == "Card" => true,
            Component::ListItem(_) if type_name == "ListItem" => true,
            Component::EmptyState(_) if type_name == "EmptyState" => true,
            Component::Stack(_) if type_name == "Stack" => true,
            Component::Form(_) if type_name == "Form" => true,
            Component::Page(_) if type_name == "Page" => true,
            Component::TextArea(_) if type_name == "TextArea" => true,
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
        Component::EmptyState(inner) => inner.children.iter().collect(),
        Component::Form(inner) => inner.children.iter().collect(),
        Component::Page(inner) => inner.children.iter().collect(),
        Component::ListItem(inner) => inner.children.iter().collect(),
        Component::Group(inner) => inner.children.iter().collect(),
        _ => Vec::new(),
    }
}

fn sample_payload() -> String {
    json!({
        "items": [
            {"icon": "clock-circle", "title": "Calme après 22 h", "subtitle": "Merci pour le voisinage"},
            {"icon": "x", "title": "Logement non-fumeur", "subtitle": "Terrasse autorisée"},
            {"icon": "users", "title": "Pas de fête ni d'événement", "subtitle": ""},
            {"icon": "check-circle", "title": "Animaux bienvenus", "subtitle": "Prévenez-nous"}
        ]
    })
    .to_string()
}

#[test]
#[serial]
fn home_card_renders_empty_without_content() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&["core.storage"])
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "EmptyState"));
        });
}

#[test]
#[serial]
fn home_card_renders_list_items_with_content() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&["core.storage"])
        .run(|ctx| {
            save_content(
                ctx.clone(),
                SaveContentArgs {
                    items: Vec::new(),
                    content_fr: sample_payload(),
                    content_en: sample_payload(),
                },
            )
            .expect("save");
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "ListItem"));
        });
}

#[test]
#[serial]
fn explore_detail_renders_full_list() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&["core.storage"])
        .run(|ctx| {
            save_content(
                ctx.clone(),
                SaveContentArgs {
                    items: Vec::new(),
                    content_fr: sample_payload(),
                    content_en: sample_payload(),
                },
            )
            .expect("save");
            let surface = render_explore_detail(ctx);
            assert!(contains_component_type(&surface, "Stack"));
            assert!(contains_component_type(&surface, "ListItem"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("Calme après 22 h"));
        });
}

#[test]
#[serial]
fn get_content_returns_saved_items() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&["core.storage"])
        .run(|ctx| {
            save_content(
                ctx.clone(),
                SaveContentArgs {
                    items: Vec::new(),
                    content_fr: sample_payload(),
                    content_en: String::new(),
                },
            )
            .expect("save");
            let view = get_content(
                ctx,
                GetContentArgs {
                    locale: Some("fr-FR".into()),
                },
            )
            .expect("get");
            assert_eq!(view.items.len(), 4);
            assert_eq!(view.items[0].title, "Calme après 22 h");
        });
}

#[test]
#[serial]
fn seed_row_shape_matches_entity() {
    let now = Utc::now();
    let row = RulesContent {
        id: Uuid::new_v4(),
        content_fr: sample_payload(),
        content_en: String::new(),
        created_at: now,
        updated_at: now,
    };
    assert!(!row.content_fr.is_empty());
}
