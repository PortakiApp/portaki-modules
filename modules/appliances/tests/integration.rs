//! Integration-style unit tests with `portaki-test-utils`.

use serial_test::serial;

use portaki_sdk::sdui::component::Component;
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::{MockContext, Property};
use serde_json::json;

use appliances::{
    get_content, render_explore_detail, render_explore_item, render_home_card, reset_test_store,
    save_content, GetContentArgs, SaveContentArgs,
};

fn contains_component_type(surface: &Surface, type_name: &str) -> bool {
    fn walk(node: &Component, type_name: &str) -> bool {
        let matches = match node {
            Component::Card(_) if type_name == "Card" => true,
            Component::ListItem(_) if type_name == "ListItem" => true,
            Component::Pressable(_) if type_name == "Pressable" => true,
            Component::EmptyState(_) if type_name == "EmptyState" => true,
            Component::Stack(_) if type_name == "Stack" => true,
            Component::InfoBanner(_) if type_name == "InfoBanner" => true,
            Component::Text(_) if type_name == "Text" => true,
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
        Component::Pressable(inner) => inner.children.iter().collect(),
        Component::Group(inner) => inner.children.iter().collect(),
        _ => Vec::new(),
    }
}

fn sample_payload() -> String {
    json!({
        "safety_notice": "Coupez l'eau en cas de fuite.",
        "devices": [
            {
                "id": "tv",
                "icon": "📺",
                "title": "Télévision",
                "subtitle": "Salon · Samsung 55\"",
                "steps": ["Allumez avec la télécommande noire.", "HDMI 1 pour Apple TV."],
                "tip": "Télécommande sur le meuble TV."
            },
            {
                "id": "washer",
                "icon": "🌀",
                "title": "Lave-linge",
                "subtitle": "Salle de bain · Bosch",
                "steps": ["Chargez le linge.", "ECO 30° puis Départ."],
                "tip": "Pas de machine après 21 h."
            }
        ]
    })
    .to_string()
}

#[test]
#[serial]
fn home_card_empty_without_content() {
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
fn home_card_and_detail_with_content() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&["core.storage"])
        .run(|ctx| {
            save_content(
                ctx.clone(),
                SaveContentArgs {
                    devices: Vec::new(),
                    safety_notice_fr: String::new(),
                    safety_notice_en: String::new(),
                    content_fr: sample_payload(),
                    content_en: sample_payload(),
                },
            )
            .expect("save");
            let card = render_home_card(ctx.clone());
            assert!(contains_component_type(&card, "Card"));
            assert!(contains_component_type(&card, "ListItem"));
            let detail = render_explore_detail(ctx.clone());
            assert!(contains_component_type(&detail, "ListItem"));
            let item = render_explore_item(ctx);
            assert!(contains_component_type(&item, "Text"));
            let json = serde_json::to_string(&item).expect("json");
            assert!(json.contains("Télévision"));
        });
}

#[test]
#[serial]
fn get_content_returns_devices() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&["core.storage"])
        .run(|ctx| {
            save_content(
                ctx.clone(),
                SaveContentArgs {
                    devices: Vec::new(),
                    safety_notice_fr: String::new(),
                    safety_notice_en: String::new(),
                    content_fr: sample_payload(),
                    content_en: String::new(),
                },
            )
            .expect("save");
            let view = get_content(
                ctx,
                GetContentArgs {
                    locale: Some("fr-FR".into()),
                    device_id: None,
                },
            )
            .expect("get");
            assert_eq!(view.devices.len(), 2);
            assert!(view.safety_notice.contains("eau"));
        });
}
