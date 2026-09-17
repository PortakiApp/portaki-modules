//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use facility_hours::{
    get_config, render_explore_detail, render_home_card, update_config, UpdateConfigArgs,
};
use portaki_test_utils::{MockContext, SurfaceAssertions};
use serde_json::json;

fn sample_config_bytes() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "facilities_json": r#"[{"id":"checkin","title":{"fr":"Arrivée","en":"Check-in"},"hours":"à partir de 16:00","lines":[]},{"id":"pool","title":{"fr":"Piscine","en":"Pool"},"hours":"08:00 – 20:00","lines":[{"fr":"Maillot obligatoire","en":"Swimwear required"}]}]"#,
        "general_note": "Horaires indicatifs"
    }))
    .expect("config json")
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
        .with_kv("config", sample_config_bytes())
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
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_explore_detail(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
            assert!(SurfaceAssertions::new(&surface).contains_type("InfoBanner"));
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
                    facilities: Vec::new(),
                    facilities_json: String::new(),
                    general_note: "note".into(),
                },
            )
            .expect("ok");
            assert_eq!(get_config(ctx).expect("cfg").general_note.get("fr"), "note");
        });
}
