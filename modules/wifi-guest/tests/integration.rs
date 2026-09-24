//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use portaki_test_utils::{MockContext, SurfaceAssertions};
use serde_json::json;
use wifi_guest::{
    get_config, publish_readiness, render_explore_detail, render_home_card, render_host_main,
    update_config, UpdateConfigArgs,
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

#[test]
#[serial]
fn home_card_renders_empty_without_config() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
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
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("KeyValue"));
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
            assert!(SurfaceAssertions::new(&surface).contains_type("InfoBanner"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Button"));
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
            assert!(SurfaceAssertions::new(&surface).contains_type("InfoBanner"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Field"));
            assert!(!SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(json.contains("i18n:host.ssid.label"));
            assert!(json.contains("i18n:host.connectionSteps.label"));
            assert!(
                json.contains("\"tone\":\"warning\"") || json.contains("\"tone\": \"warning\"")
            );
        });
}

#[test]
#[serial]
fn publish_readiness_requires_ssid_only() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let items = publish_readiness(ctx).expect("publishReadiness").items;
            assert!(items.iter().all(|item| !item.ok));
        });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let items = publish_readiness(ctx).expect("publishReadiness").items;
            assert!(items.iter().all(|item| item.ok));
        });
}
