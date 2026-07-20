//! Integration-style unit tests with `portaki-test-utils`.

use chrono::{TimeZone, Utc};
use serial_test::serial;

use access_guide::{
    get_config, render_explore_detail, render_home_card, update_config, MethodFields,
    PrimaryMethod, RevealPolicy, UpdateConfigArgs,
};
use portaki_sdk::context::StayContext;
use portaki_sdk::host::with_host;
use portaki_sdk::sdui::component::Component;
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::MockContext;
use serde_json::json;
use uuid::Uuid;

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

fn always_reveal_config_bytes() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "primary_method": "keybox",
        "method": {
            "kind": "keybox",
            "location": "À droite de la porte",
            "code": "4821"
        },
        "building_access": { "gate_code": "A17B" },
        "parking": { "info": "Rue A", "map_url": "https://maps.example.com" },
        "arrival": {
            "address": "Ch. des Douaniers",
            "arrival_video_url": "https://video.example.com",
            "global_note": "Sonnette à gauche",
            "steps": [{"id":"1","kind":"parking","title":{"fr":"Se garer","en":"Park"},"detail":{"fr":"Place résident","en":"Resident spot"}}]
        },
        "reveal_policy": "always"
    }))
    .expect("config json")
}

fn smart_lock_config_bytes(provider: Option<&str>) -> Vec<u8> {
    let mut cfg = json!({
        "primary_method": "smart_lock",
        "method": {
            "kind": "smart_lock",
            "instructions": "Appuyer sur unlock",
            "manual_code": "9999"
        },
        "arrival": { "address": "1 rue Test" },
        "reveal_policy": "always"
    });
    if let Some(id) = provider {
        cfg["smart_lock_provider_module_id"] = json!(id);
    }
    serde_json::to_vec(&cfg).expect("config json")
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
            Component::Map(_) if type_name == "Map" => true,
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
fn home_card_masks_secrets_without_stay() {
    // Legacy config defaults to day_before_16h; no checkin → fail-safe lock.
    MockContext::guest()
        .with_capabilities(&["core.storage"])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "KeyValue"));
            assert!(contains_component_type(&surface, "Button"));
            assert!(contains_component_type(&surface, "Map"));
            assert!(contains_component_type(&surface, "InfoBanner"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("navigate"));
            assert!(json.contains("access-guide"));
            assert!(!json.contains("4821"));
            assert!(!json.contains("A17B"));
            assert!(json.contains("••••••"));
            assert!(json.contains("\"mono\":true") || json.contains("\"mono\": true"));
        });
}

#[test]
#[serial]
fn home_card_reveals_secrets_when_policy_always() {
    MockContext::guest()
        .with_capabilities(&["core.storage"])
        .with_kv("config", always_reveal_config_bytes())
        .run(|ctx| {
            let surface = render_home_card(ctx);
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("4821"));
            assert!(json.contains("A17B"));
            assert!(!json.contains("••••••"));
        });
}

#[test]
#[serial]
fn detail_has_steps_and_video() {
    MockContext::guest()
        .with_capabilities(&["core.storage"])
        .with_kv("config", always_reveal_config_bytes())
        .run(|ctx| {
            let surface = render_explore_detail(ctx);
            assert!(contains_component_type(&surface, "ListItem"));
            assert!(contains_component_type(&surface, "Badge"));
            assert!(contains_component_type(&surface, "Link"));
        });
}

#[test]
#[serial]
fn smart_lock_provider_emits_unlock_commands_when_revealed() {
    MockContext::guest()
        .with_capabilities(&["core.storage"])
        .with_kv("config", smart_lock_config_bytes(Some("nuki")))
        .run(|ctx| {
            let surface = render_explore_detail(ctx);
            let json = serde_json::to_string(&surface).expect("json");
            assert!(
                json.contains("\"type\":\"command\"") || json.contains("\"type\": \"command\"")
            );
            assert!(json.contains("nuki"));
            assert!(json.contains("unlock"));
            assert!(json.contains("getGuestCredential"));
            assert!(json.contains("9999"));
        });
}

#[test]
#[serial]
fn smart_lock_without_provider_shows_manual_fallback_only() {
    MockContext::guest()
        .with_capabilities(&["core.storage"])
        .with_kv("config", smart_lock_config_bytes(None))
        .run(|ctx| {
            let surface = render_explore_detail(ctx);
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("9999"));
            assert!(json.contains("Appuyer sur unlock"));
            assert!(!json.contains("getGuestCredential"));
        });
}

#[test]
#[serial]
fn smart_lock_provider_hides_cta_when_not_revealed() {
    let cfg = serde_json::to_vec(&json!({
        "primary_method": "smart_lock",
        "method": {
            "kind": "smart_lock",
            "manual_code": "9999"
        },
        "arrival": { "address": "1 rue Test" },
        "reveal_policy": "at_checkin",
        "smart_lock_provider_module_id": "nuki"
    }))
    .expect("json");

    let (mut ctx, host) = MockContext::guest()
        .with_capabilities(&["core.storage"])
        .with_kv("config", cfg)
        .build();
    ctx.timezone = "Europe/Paris".into();
    ctx.property.timezone = "Europe/Paris".into();
    ctx.stay = Some(StayContext {
        stay_id: Uuid::nil(),
        checkin_at: Some(
            Utc.with_ymd_and_hms(2099, 1, 1, 15, 0, 0)
                .single()
                .expect("dt"),
        ),
        checkout_at: None,
    });

    with_host(host, ctx.clone(), || {
        let surface = render_home_card(ctx);
        let json = serde_json::to_string(&surface).expect("json");
        assert!(!json.contains("unlock"));
        assert!(!json.contains("getGuestCredential"));
        assert!(!json.contains("9999"));
        assert!(json.contains("••••••"));
    });
}

#[test]
#[serial]
fn update_config_legacy_args_migrate_to_new_shape() {
    MockContext::host()
        .with_capabilities(&["core.storage"])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    address: "Rue X".into(),
                    gate_code: "1".into(),
                    ..UpdateConfigArgs::default()
                },
            )
            .expect("ok");
            let cfg = get_config(ctx).expect("cfg");
            assert_eq!(cfg.arrival.address, "Rue X");
            assert_eq!(cfg.primary_method, PrimaryMethod::DoorCode);
            assert_eq!(cfg.reveal_policy, RevealPolicy::DayBefore16h);
            match &cfg.method {
                MethodFields::DoorCode { code, .. } => assert_eq!(code, "1"),
                other => panic!("expected DoorCode, got {other:?}"),
            }
        });
}

#[test]
#[serial]
fn load_legacy_kv_migrates_keybox_primary() {
    MockContext::host()
        .with_capabilities(&["core.storage"])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let cfg = get_config(ctx).expect("cfg");
            assert_eq!(cfg.primary_method, PrimaryMethod::Keybox);
            assert_eq!(cfg.keybox_code(), Some("4821"));
            assert_eq!(
                cfg.building_access
                    .as_ref()
                    .and_then(|b| b.gate_code.as_deref()),
                Some("A17B")
            );
            assert_eq!(cfg.reveal_policy, RevealPolicy::DayBefore16h);
            assert_eq!(cfg.parse_steps().len(), 1);
        });
}
