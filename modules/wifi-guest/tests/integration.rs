//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use portaki_sdk::sdui::component::Component;
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::MockContext;
use serde_json::json;
use wifi_guest::{
    get_config, render_explore_detail, render_home_card, render_host_main, update_config,
    UpdateConfigArgs,
};

fn sample_config_bytes() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "ssid": "Islette_Guest",
        "password": "soleil2026",
        "hint": "Prefer 5 GHz",
        "reveal_policy": "day_before_16h"
    }))
    .expect("config json")
}

fn always_reveal_config_bytes() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "ssid": "Islette_Guest",
        "password": "soleil2026",
        "reveal_policy": "always"
    }))
    .expect("config json")
}

fn contains_component_type(surface: &Surface, type_name: &str) -> bool {
    fn walk(node: &Component, type_name: &str) -> bool {
        let matches = match node {
            Component::Card(_) if type_name == "Card" => true,
            Component::KeyValue(_) if type_name == "KeyValue" => true,
            Component::InfoBanner(_) if type_name == "InfoBanner" => true,
            Component::EmptyState(_) if type_name == "EmptyState" => true,
            Component::Button(_) if type_name == "Button" => true,
            Component::Stack(_) if type_name == "Stack" => true,
            Component::Field(_) if type_name == "Field" => true,
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
        Component::EmptyState(inner) => inner.children.iter().collect(),
        Component::Field(inner) => inner.children.iter().collect(),
        Component::Form(inner) => inner.children.iter().collect(),
        Component::Page(inner) => inner.children.iter().collect(),
        _ => Vec::new(),
    }
}

#[test]
#[serial]
fn home_card_renders_empty_without_config() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "EmptyState"));
        });
}

#[test]
#[serial]
fn home_card_renders_with_config_and_masks_password() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "KeyValue"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("Islette_Guest"));
            assert!(json.contains("i18n:nav.wifi-guest"));
            assert!(!json.contains("soleil2026"));
            assert!(json.contains("••••••"));
        });
}

#[test]
#[serial]
fn detail_shows_security_banner_and_copy_when_revealed() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", always_reveal_config_bytes())
        .run(|ctx| {
            let surface = render_explore_detail(ctx);
            assert!(contains_component_type(&surface, "InfoBanner"));
            assert!(contains_component_type(&surface, "Button"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("soleil2026"));
            assert!(json.contains("\"type\":\"copy\"") || json.contains("\"type\": \"copy\""));
        });
}

#[test]
#[serial]
fn update_config_roundtrip() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    ssid: "TestNet".into(),
                    password: "hunter2".into(),
                    hint: "Guest only".into(),
                    connection_steps: "Join then open the captive page.".into(),
                    reveal_policy: wifi_guest::RevealPolicy::Always,
                },
            )
            .expect("updateConfig");
            let config = get_config(ctx).expect("getConfig");
            assert_eq!(config.ssid, "TestNet");
            assert_eq!(config.password, "hunter2");
            assert_eq!(config.hint.as_deref(), Some("Guest only"));
            assert_eq!(
                config.connection_steps.as_deref(),
                Some("Join then open the captive page.")
            );
            assert_eq!(config.reveal_policy, wifi_guest::RevealPolicy::Always);
        });
}

#[test]
#[serial]
fn update_config_keeps_password_when_blank() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    ssid: "Renamed".into(),
                    password: String::new(),
                    hint: String::new(),
                    connection_steps: String::new(),
                    reveal_policy: wifi_guest::RevealPolicy::DayBefore16h,
                },
            )
            .expect("updateConfig");
            let config = get_config(ctx).expect("getConfig");
            assert_eq!(config.ssid, "Renamed");
            assert_eq!(config.password, "soleil2026");
        });
}

#[test]
#[serial]
fn host_main_is_flat_drawer_form_without_cards() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", always_reveal_config_bytes())
        .run(|ctx| {
            let surface = render_host_main(ctx);
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(contains_component_type(&surface, "InfoBanner"));
            assert!(contains_component_type(&surface, "Field"));
            assert!(!contains_component_type(&surface, "Card"));
            assert!(json.contains("i18n:host.ssid.label"));
            assert!(json.contains("i18n:host.connectionSteps.label"));
            assert!(
                json.contains("\"tone\":\"warning\"") || json.contains("\"tone\": \"warning\"")
            );
        });
}
