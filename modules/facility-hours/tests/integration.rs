//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use facility_hours::{render_explore_detail, render_home_card, render_host_main};
use portaki_sdk::host::module::ModuleStatus;
use portaki_test_utils::{MockContext, SurfaceAssertions};
use serde_json::{json, Value};

#[path = "../../../support/config_form.rs"]
mod config_form;
#[path = "../../../support/config_save.rs"]
mod config_save;

const EMISSIONS: &str = concat!(env!("OUT_DIR"), "/portaki-emissions");

fn sample_config() -> Value {
    json!({
        "facilities": [
            { "id": "pool", "title": { "fr": "Piscine", "en": "Pool" }, "hours": "08:00 – 20:00", "lines": { "fr": "Maillot obligatoire", "en": "Swimwear required" } },
            { "title": "Accueil", "hours": "à partir de 16:00" }
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
            assert!(
                SurfaceAssertions::new(&render_home_card(ctx).expect("home card"))
                    .contains_type("EmptyState")
            );
        });
}

#[test]
#[serial]
fn home_card_uses_key_value_and_page_overlay() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("home card");
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
            let surface = render_explore_detail(ctx).expect("detail");
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

/// A host writing in English: the French title, lines and note stay, and so does the id; rows
/// keep their place.
#[test]
#[serial]
fn a_save_in_english_keeps_the_french() {
    assert_eq!(
        config_save::localized_paths(EMISSIONS),
        [
            "facilities.lines",
            "facilities.note",
            "facilities.title",
            "general_note"
        ]
    );
    let stored = json!({
        "facilities": [
            { "id": "pool", "title": { "fr": "Piscine", "en": "Pool" }, "hours": "9 h – 20 h",
              "lines": { "fr": "Tous les jours\nEnfants accompagnés", "en": "Every day\nChildren with an adult" },
              "note": { "fr": "Bonnet", "en": "Cap" } },
            { "title": "", "hours": "" },
            { "id": "spa", "title": { "fr": "Spa" }, "hours": "10 h – 19 h" }
        ],
        "general_note": { "fr": "Horaires indicatifs", "en": "Indicative hours" }
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|mut ctx| {
            ctx.locale = "en-US".into();
            let surface = render_host_main(ctx).expect("host main");
            let sent = config_save::form_args(&surface);
            // Stored order, the blank row where it was; ids on the filled rows only.
            assert_eq!(sent["facilities"][0]["id"], "pool");
            assert_eq!(sent["facilities"][0]["title"], "Pool");
            assert_eq!(
                sent["facilities"][0]["lines"],
                "Every day\nChildren with an adult"
            );
            assert_eq!(sent["facilities"][0]["note"], "Cap");
            assert!(sent["facilities"][1].get("id").is_none());
            assert_eq!(sent["facilities"][2]["id"], "spa");
            assert_eq!(sent["facilities"].as_array().unwrap().len(), 6);
            assert_eq!(sent["general_note"], "Indicative hours");

            let saved = config_save::save(EMISSIONS, &surface, &stored, "en");
            assert_eq!(saved["facilities"][0], stored["facilities"][0]);
            assert_eq!(saved["facilities"][2]["title"]["fr"], "Spa");
            assert_eq!(saved["general_note"], stored["general_note"]);
        });
}
