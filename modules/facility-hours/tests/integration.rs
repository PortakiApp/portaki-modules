//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use facility_hours::{render_explore_detail, render_home_card, render_host_main};
use portaki_test_utils::{MockContext, SurfaceAssertions};
use serde_json::{json, Value};

#[path = "../../../support/config_form.rs"]
mod config_form;

fn sample_config() -> Value {
    json!({
        "facilities": [
            { "id": "pool", "title": { "fr": "Piscine", "en": "Pool" }, "hours": "08:00 – 20:00", "lines": [{ "fr": "Maillot obligatoire", "en": "Swimwear required" }] },
            { "name": "Accueil", "hours": "à partir de 16:00" }
        ],
        "general_note": "Horaires indicatifs"
    })
}

#[test]
#[serial]
fn home_card_empty_without_config() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            assert!(SurfaceAssertions::new(&render_home_card(ctx)).contains_type("EmptyState"));
        });
}

#[test]
#[serial]
fn home_card_uses_key_value_and_page_overlay() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("KeyValue"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("explore.detail"));
            assert!(json.contains("bottomSheet"));
        });
}

#[test]
#[serial]
fn detail_enriched_list() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
            assert!(SurfaceAssertions::new(&surface).contains_type("InfoBanner"));
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
            assert!(json.contains("Piscine"));
            assert!(json.contains("Accueil"));
            assert!(json.contains("Horaires indicatifs"));
        });
}
