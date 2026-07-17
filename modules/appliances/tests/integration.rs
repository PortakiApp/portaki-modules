//! Integration-style unit tests with `portaki-test-utils`.

use serial_test::serial;

use portaki_sdk::sdui::component::Component;
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::{MockContext, Property};
use serde_json::json;

use appliances::{
    get_content, render_explore_detail, render_explore_item, render_home_card, reset_test_store,
    save_appliance, ApplianceStatus, GetContentArgs, SaveApplianceArgs,
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
            Component::RichText(_) if type_name == "RichText" => true,
            Component::Link(_) if type_name == "Link" => true,
            Component::Eyebrow(_) if type_name == "Eyebrow" => true,
            Component::Button(_) if type_name == "Button" => true,
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

fn seed_two_devices(ctx: portaki_sdk::prelude::Context) {
    save_appliance(
        ctx.clone(),
        SaveApplianceArgs {
            id: Some("tv".into()),
            name: "Télévision".into(),
            emoji: "📺".into(),
            description: json!({
                "type": "doc",
                "content": [{
                    "type": "bulletList",
                    "content": [{
                        "type": "listItem",
                        "content": [{
                            "type": "paragraph",
                            "content": [{ "type": "text", "text": "Allumez avec la télécommande." }]
                        }]
                    }]
                }]
            })
            .to_string(),
            featured: true,
            order: Some(0),
            location: "Salon · Samsung 55\"".into(),
            manual_url: "https://example.com/tv-manual".into(),
            safety_note: String::new(),
            status: ApplianceStatus::Active,
        },
    )
    .expect("save tv");
    save_appliance(
        ctx,
        SaveApplianceArgs {
            id: Some("washer".into()),
            name: "Lave-linge".into(),
            emoji: "🌀".into(),
            description: json!({
                "type": "doc",
                "content": [{
                    "type": "paragraph",
                    "content": [{ "type": "text", "text": "ECO 30°" }]
                }]
            })
            .to_string(),
            featured: false,
            order: Some(1),
            location: "Salle de bain · Bosch".into(),
            manual_url: String::new(),
            safety_note: "Pas de machine après 21 h.".into(),
            status: ApplianceStatus::Active,
        },
    )
    .expect("save washer");
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
fn home_card_featured_only_and_detail_list() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&["core.storage"])
        .run(|ctx| {
            seed_two_devices(ctx.clone());
            let card = render_home_card(ctx.clone());
            assert!(contains_component_type(&card, "Card"));
            assert!(contains_component_type(&card, "ListItem"));
            let card_json = serde_json::to_string(&card).expect("json");
            assert!(card_json.contains("Télévision"));
            assert!(!card_json.contains("Lave-linge"));
            assert!(card_json.contains("\"type\":\"navigate\""));
            assert!(card_json.contains("\"to\":\"appliances\""));
            assert!(card_json.contains("appliances/tv"));

            let detail = render_explore_detail(ctx.clone());
            assert!(contains_component_type(&detail, "Card"));
            assert!(contains_component_type(&detail, "ListItem"));
            let detail_json = serde_json::to_string(&detail).expect("json");
            assert!(detail_json.contains("Télévision"));
            assert!(detail_json.contains("Lave-linge"));
            assert!(detail_json.contains("appliances/washer"));
        });
}

#[test]
#[serial]
fn explore_item_uses_device_id_and_howto_steps() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&["core.storage"])
        .run(|ctx| {
            seed_two_devices(ctx.clone());

            let mut tv_ctx = ctx.clone();
            tv_ctx.input = json!({ "deviceId": "tv" });
            let tv = render_explore_item(tv_ctx);
            assert!(contains_component_type(&tv, "ListItem"));
            assert!(contains_component_type(&tv, "Eyebrow"));
            assert!(contains_component_type(&tv, "Button"));
            assert!(contains_component_type(&tv, "Link"));
            let tv_json = serde_json::to_string(&tv).expect("json");
            assert!(tv_json.contains("Télévision"));
            assert!(tv_json.contains("Allumez avec la télécommande."));
            assert!(tv_json.contains("https://example.com/tv-manual"));
            assert!(tv_json.contains("openHostChat"));
            assert!(!tv_json.contains("Lave-linge"));

            let mut washer_ctx = ctx.clone();
            washer_ctx.input = json!({ "deviceId": "washer" });
            let washer = render_explore_item(washer_ctx);
            assert!(contains_component_type(&washer, "InfoBanner"));
            let washer_json = serde_json::to_string(&washer).expect("json");
            assert!(washer_json.contains("Lave-linge"));
            assert!(washer_json.contains("Pas de machine après 21 h."));
            assert!(washer_json.contains("ECO 30"));
        });
}

#[test]
#[serial]
fn explore_item_missing_device_id_is_not_found() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&["core.storage"])
        .run(|ctx| {
            seed_two_devices(ctx.clone());
            let item = render_explore_item(ctx);
            let json = serde_json::to_string(&item).expect("json");
            assert!(json.contains("explore.item.notFound"));
            assert!(!json.contains("Télévision"));
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
            seed_two_devices(ctx.clone());
            let view = get_content(
                ctx,
                GetContentArgs {
                    locale: Some("fr-FR".into()),
                    device_id: None,
                },
            )
            .expect("get");
            assert_eq!(view.devices.len(), 2);
            assert!(view.devices.iter().any(|d| d.name.contains("Télévision")));
        });
}

#[test]
#[serial]
fn migrates_legacy_payload_on_read() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&["core.storage"])
        .run(|ctx| {
            let legacy = json!({
                "safety_notice": "Coupez l'eau en cas de fuite.",
                "devices": [{
                    "id": "tv",
                    "icon": "📺",
                    "title": "Télévision",
                    "subtitle": "Salon",
                    "steps": ["Allumez", "HDMI 1"],
                    "tip": "Remote on stand"
                }]
            })
            .to_string();
            appliances::store_save_legacy_for_tests(legacy).expect("seed legacy");
            let view = get_content(
                ctx.clone(),
                GetContentArgs {
                    locale: Some("fr-FR".into()),
                    device_id: None,
                },
            )
            .expect("get");
            assert_eq!(view.devices.len(), 1);
            assert_eq!(view.devices[0].name, "Télévision");
            assert_eq!(view.devices[0].emoji, "📺");
            assert!(!view.devices[0].featured);
            assert!(view.devices[0].description.contains("bulletList"));
            let card = render_home_card(ctx);
            // featured=false after migration → empty featured card children, still Card
            assert!(contains_component_type(&card, "Card"));
        });
}
