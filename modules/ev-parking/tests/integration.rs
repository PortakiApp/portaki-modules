//! Integration-style unit tests with `portaki-test-utils`.

use ev_parking::{
    email_context, render_explore_detail, render_home_card, render_host_main, EmailContextArgs,
};
use portaki_sdk::capability;
use portaki_sdk::prelude::EmailTemplateKey;
use serial_test::serial;

use portaki_sdk::host::module::ModuleStatus;
use portaki_test_utils::{MockContext, SurfaceAssertions};
use serde_json::{json, Value};

#[path = "../../../support/config_form.rs"]
mod config_form;

fn sample_config() -> Value {
    json!({
        "spot_label": "P2 / Place 14",
        "charger_pin": "4821",
        "parking_code": "1234",
        "map_url": "https://maps.example/parking",
        "instructions": "Left entrance after the barrier",
        "reveal_policy": "day_before_16h"
    })
}

fn always_reveal_config() -> Value {
    json!({
        "spot_label": "P2 / Place 14",
        "charger_pin": "4821",
        "parking_code": "1234",
        "reveal_policy": "always"
    })
}

#[test]
#[serial]
fn home_card_renders_empty_without_config() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("home card");
            assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
        });
}

#[test]
#[serial]
fn home_card_renders_with_config_and_masks_secrets() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("home card");
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
        .with_config(&always_reveal_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("detail");
            assert!(SurfaceAssertions::new(&surface).contains_type("Button"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("4821"));
            assert!(json.contains("1234"));
            assert!(json.contains("\"type\":\"copy\"") || json.contains("\"type\": \"copy\""));
        });
}

#[test]
#[serial]
fn the_host_form_sends_the_declared_keys() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            config_form::assert_form_matches_config(
                concat!(env!("OUT_DIR"), "/portaki-emissions"),
                &surface,
                &[],
            );
            // The codes are never sent back to the form: blank keeps them.
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("P2 / Place 14"));
            assert!(!json.contains("4821"));
            assert!(!json.contains("1234"));
        });
}

#[test]
#[serial]
fn email_context_returns_ev_parking_spot_for_arrival() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
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

/// The SDK renders the inactive state; the surface itself is not called.
#[test]
#[serial]
fn an_inactive_module_shows_the_sdk_state() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_module_status(ModuleStatus {
            active: false,
            workspace_enabled: true,
            incomplete: false,
            requires_config: true,
            missing_required_keys: Vec::new(),
        })
        .run(|ctx| {
            let surface = portaki_sdk::guest_shell::render(ctx, "home.card", render_home_card);
            assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
        });
}
