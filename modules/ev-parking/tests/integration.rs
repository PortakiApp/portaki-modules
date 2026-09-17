//! Integration-style unit tests with `portaki-test-utils`.

use ev_parking::{
    email_context, get_config, render_explore_detail, render_home_card, update_config,
    EmailContextArgs, UpdateConfigArgs,
};
use portaki_sdk::capability;
use portaki_sdk::prelude::EmailTemplateKey;
use serial_test::serial;

use portaki_test_utils::{MockContext, SurfaceAssertions};
use serde_json::json;

fn sample_config_bytes() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "spot_label": "P2 / Place 14",
        "charger_pin": "4821",
        "parking_code": "1234",
        "map_url": "https://maps.example/parking",
        "instructions": "Left entrance after the barrier",
        "reveal_policy": "day_before_16h"
    }))
    .expect("config json")
}

fn always_reveal_config_bytes() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "spot_label": "P2 / Place 14",
        "charger_pin": "4821",
        "parking_code": "1234",
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
fn home_card_renders_with_config_and_masks_secrets() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("KeyValue"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Link"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("P2 / Place 14"));
            assert!(json.contains("i18n:nav.ev-parking"));
            assert!(json.contains("https://maps.example/parking"));
            assert!(!json.contains("4821"));
            assert!(!json.contains("1234"));
            assert!(json.contains("••••••"));
        });
}

#[test]
#[serial]
fn detail_shows_copy_buttons_when_revealed() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", always_reveal_config_bytes())
        .run(|ctx| {
            let surface = render_explore_detail(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("Button"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("4821"));
            assert!(json.contains("1234"));
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
                    spot_label: "B1 / 3".into(),
                    charger_pin: "9999".into(),
                    parking_code: "4321".into(),
                    map_url: "https://maps.test".into(),
                    instructions: "Ring the bell".into(),
                    reveal_policy: ev_parking::RevealPolicy::Always,
                },
            )
            .expect("updateConfig");
            let config = get_config(ctx).expect("getConfig");
            assert_eq!(config.spot_label, "B1 / 3");
            assert_eq!(config.charger_pin, "9999");
            assert_eq!(config.parking_code, "4321");
            assert_eq!(config.map_url.as_deref(), Some("https://maps.test"));
            assert_eq!(config.instructions.as_deref(), Some("Ring the bell"));
            assert_eq!(config.reveal_policy, ev_parking::RevealPolicy::Always);
        });
}

#[test]
#[serial]
fn update_config_keeps_secrets_when_blank() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    spot_label: "Renamed".into(),
                    charger_pin: String::new(),
                    parking_code: String::new(),
                    map_url: String::new(),
                    instructions: String::new(),
                    reveal_policy: ev_parking::RevealPolicy::DayBefore16h,
                },
            )
            .expect("updateConfig");
            let config = get_config(ctx).expect("getConfig");
            assert_eq!(config.spot_label, "Renamed");
            assert_eq!(config.charger_pin, "4821");
            assert_eq!(config.parking_code, "1234");
        });
}

#[test]
#[serial]
fn email_context_returns_ev_parking_spot_for_arrival() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let response = email_context(
                ctx,
                EmailContextArgs {
                    template_key: Some(EmailTemplateKey::ArrivalDay),
                    locale: None,
                    ..Default::default()
                },
            )
            .expect("emailContext");
            assert_eq!(response.ev_parking_spot.as_deref(), Some("P2 / Place 14"));
        });
}
