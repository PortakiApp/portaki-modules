//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use emergency_contacts::{render_explore_detail, render_home_card, render_host_main};
use portaki_sdk::host::module::ModuleStatus;
use portaki_test_utils::{MockContext, SurfaceAssertions};
use serde_json::{json, Value};

#[path = "../../../support/config_form.rs"]
mod config_form;

fn sample_config() -> Value {
    json!({
        "contacts": [
            { "id": "samu", "label": { "fr": "SAMU", "en": "SAMU" }, "phone": "15" },
            { "label": "Pompiers", "phone": "18" }
        ],
        "host_visible_phone": "+33 6 12 34 56 78"
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
fn home_card_renders_contacts() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("home card");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Pressable"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("bottomSheet"));
        });
}

#[test]
#[serial]
fn detail_includes_emergency_banner() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("detail");
            assert!(SurfaceAssertions::new(&surface).contains_type("InfoBanner"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Link"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("Pompiers"));
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
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("+33 6 12 34 56 78"));
            assert!(json.contains("Pompiers"));
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
