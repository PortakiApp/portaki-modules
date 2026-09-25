//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use portaki_test_utils::{MockContext, Property, SurfaceAssertions};
use serde_json::json;

use appliances::{
    get_content, render_explore_detail, render_explore_item, render_home_card, reset_test_store,
    save_appliance, ApplianceStatus, GetContentArgs, SaveApplianceArgs,
};

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
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("render");
            assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
        });
}

#[test]
#[serial]
fn home_card_featured_only_and_detail_list() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            seed_two_devices(ctx.clone());
            let card = render_home_card(ctx.clone()).expect("render");
            assert!(SurfaceAssertions::new(&card).contains_type("Card"));
            assert!(SurfaceAssertions::new(&card).contains_type("ListItem"));
            let card_json = serde_json::to_string(&card).expect("json");
            assert!(card_json.contains("Télévision"));
            assert!(!card_json.contains("Lave-linge"));
            assert!(card_json.contains("\"type\":\"openOverlay\""));
            assert!(card_json.contains("explore.detail"));
            assert!(card_json.contains("appliances/tv"));

            let detail = render_explore_detail(ctx.clone()).expect("render");
            assert!(SurfaceAssertions::new(&detail).contains_type("Card"));
            assert!(SurfaceAssertions::new(&detail).contains_type("ListItem"));
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
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            seed_two_devices(ctx.clone());

            let mut tv_ctx = ctx.clone();
            tv_ctx.input = json!({ "deviceId": "tv" });
            let tv = render_explore_item(tv_ctx).expect("render");
            assert!(SurfaceAssertions::new(&tv).contains_type("ListItem"));
            assert!(SurfaceAssertions::new(&tv).contains_type("Eyebrow"));
            assert!(SurfaceAssertions::new(&tv).contains_type("Button"));
            assert!(SurfaceAssertions::new(&tv).contains_type("Link"));
            let tv_json = serde_json::to_string(&tv).expect("json");
            assert!(tv_json.contains("Télévision"));
            assert!(tv_json.contains("Allumez avec la télécommande."));
            assert!(tv_json.contains("https://example.com/tv-manual"));
            assert!(tv_json.contains("openHostChat"));
            assert!(!tv_json.contains("Lave-linge"));

            let mut washer_ctx = ctx.clone();
            washer_ctx.input = json!({ "deviceId": "washer" });
            let washer = render_explore_item(washer_ctx).expect("render");
            assert!(SurfaceAssertions::new(&washer).contains_type("InfoBanner"));
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
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            seed_two_devices(ctx.clone());
            let item = render_explore_item(ctx).expect("render");
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
        .with_capabilities(&[capability::core::STORAGE])
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
        .with_capabilities(&[capability::core::STORAGE])
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
            assert!(view.safety_notice.contains("Coupez l'eau"));
            let card = render_home_card(ctx).expect("render");
            // featured=false after migration → empty featured card children, still Card
            assert!(SurfaceAssertions::new(&card).contains_type("Card"));
        });
}
