//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use portaki_sdk::host::module::ModuleStatus;
use portaki_test_utils::{MockContext, SurfaceAssertions};
use serde_json::{json, Value};

use waste_recycling::{render_explore_detail, render_home_card, render_host_main};

#[path = "../../../support/config_form.rs"]
mod config_form;
#[path = "../../../support/config_save.rs"]
mod config_save;

const EMISSIONS: &str = concat!(env!("OUT_DIR"), "/portaki-emissions");

fn sample_config() -> Value {
    json!({
        "bins": [
            {
                "id": "yellow",
                "title": {"fr": "Bac jaune", "en": "Yellow bin"},
                "items": {"fr": "Emballages, plastique", "en": "Packaging, plastic"},
                "color": "#f4c020"
            },
            {
                "id": "green",
                "title": {"fr": "Bac vert", "en": "Green bin"},
                "items": {"fr": "Verre", "en": "Glass"},
                "color": "#3a8a4d"
            }
        ],
        "collection_schedule": "Mardi & vendredi matin"
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
fn home_card_renders_bins_with_config() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("home card");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ColorDotItem"));
            assert!(SurfaceAssertions::new(&surface).contains_type("InfoBanner"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("bottomSheet"));
            assert!(json.contains("explore.detail"));
        });
}

/// The stored hex of an older config becomes a named swatch on the wire — no CSS color leaves
/// the module, the booklet picks the yellow.
#[test]
#[serial]
fn bins_render_as_named_swatches() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let json = serde_json::to_string(&render_home_card(ctx).expect("home card"))
                .expect("surface json");
            assert!(json.contains(r#""swatch":"yellow""#), "{json}");
            assert!(json.contains(r#""swatch":"green""#), "{json}");
            assert!(!json.contains("#f4c020"), "{json}");
        });
}

#[test]
#[serial]
fn detail_renders_enriched_bins() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("detail");
            assert!(SurfaceAssertions::new(&surface).contains_type("Stack"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ColorDotItem"));
        });
}

#[test]
#[serial]
fn the_host_form_sends_the_declared_keys() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "bins": [
                { "id": "yellow", "title": { "fr": "Bac jaune" }, "items": { "fr": "Plastique\nCarton" }, "color": "#f4c020" },
                { "title": "Bac vert", "items": "Verre", "color": "green" }
            ],
            "collection_schedule": "Mardi"
        }))
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            config_form::assert_form_matches_config(
                concat!(env!("OUT_DIR"), "/portaki-emissions"),
                &surface,
                &[],
            );
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains(r"Plastique\nCarton"), "{json}");
            assert!(json.contains("Bac vert"));
            assert!(json.contains(r#""value":"yellow""#), "{json}");
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

/// A host writing in English: the French title, items and schedule stay, and so do the id and
/// the color; rows keep their place.
#[test]
#[serial]
fn a_save_in_english_keeps_the_french() {
    assert_eq!(
        config_save::localized_paths(EMISSIONS),
        ["bins.items", "bins.title", "collection_schedule"]
    );
    let stored = json!({
        "bins": [
            { "id": "yellow", "title": { "fr": "Bac jaune", "en": "Yellow bin" },
              "items": { "fr": "Plastique\nCarton", "en": "Plastic\nCardboard" }, "color": "yellow" },
            { "title": "", "items": "" },
            { "id": "glass", "title": { "fr": "Verre" }, "items": { "fr": "Bouteilles" }, "color": "green" }
        ],
        "collection_schedule": { "fr": "Mardi", "en": "Tuesday" }
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|mut ctx| {
            ctx.locale = "en-US".into();
            let surface = render_host_main(ctx).expect("host main");
            let sent = config_save::form_args(&surface);
            // Stored order, the blank row where it was; ids on the filled rows only.
            assert_eq!(sent["bins"][0]["id"], "yellow");
            assert_eq!(sent["bins"][0]["items"], "Plastic\nCardboard");
            assert!(sent["bins"][1].get("id").is_none());
            assert_eq!(sent["bins"][2]["id"], "glass");
            assert_eq!(sent["bins"].as_array().unwrap().len(), 6);

            let saved = config_save::save(EMISSIONS, &surface, &stored, "en");
            assert_eq!(saved["bins"][0], stored["bins"][0]);
            assert_eq!(saved["bins"][2]["title"]["fr"], "Verre");
            assert_eq!(saved["bins"][2]["items"]["fr"], "Bouteilles");
            assert_eq!(saved["collection_schedule"], stored["collection_schedule"]);
        });
}
