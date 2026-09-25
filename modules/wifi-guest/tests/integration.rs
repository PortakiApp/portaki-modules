//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use portaki_sdk::contracts::i18n::I18nText;
use serial_test::serial;

use portaki_test_utils::{MockContext, SurfaceAssertions};
use wifi_guest::{
    render_explore_detail, render_home_card, render_host_main, ModuleConfig, RevealPolicy,
};

#[path = "../../../support/config_form.rs"]
mod config_form;
#[path = "../../../support/config_save.rs"]
mod config_save;

const EMISSIONS: &str = concat!(env!("OUT_DIR"), "/portaki-emissions");

fn sample_config() -> ModuleConfig {
    ModuleConfig {
        ssid: "Islette_Guest".into(),
        password: "soleil2026".into(),
        hint: Some(I18nText::new("5 GHz conseillé", "Prefer 5 GHz")),
        connection_steps: None,
        reveal_policy: RevealPolicy::DayBefore16h,
    }
}

fn always_reveal_config() -> ModuleConfig {
    ModuleConfig {
        reveal_policy: RevealPolicy::Always,
        hint: None,
        ..sample_config()
    }
}

#[test]
#[serial]
fn home_card_renders_empty_without_config() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("guest surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
        });
}

#[test]
#[serial]
fn home_card_renders_with_config_and_masks_password() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("guest surface");
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
        .with_config(&always_reveal_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("guest surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("InfoBanner"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Button"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("soleil2026"));
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
            // The password is never sent back to the form: blank keeps it.
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("Islette_Guest"));
            assert!(!json.contains("soleil2026"));
        });
}

#[test]
#[serial]
fn host_main_is_flat_drawer_form_without_cards() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&always_reveal_config())
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
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

/// A host writing in English: the French hint and steps stay; the guest reads their language.
#[test]
#[serial]
fn a_save_in_english_keeps_the_french() {
    assert_eq!(
        config_save::localized_paths(EMISSIONS),
        ["connection_steps", "hint"]
    );
    let stored = serde_json::json!({
        "ssid": "Villa",
        "hint": { "fr": "5 GHz conseillé", "en": "Prefer 5 GHz" },
        "connection_steps": { "fr": "Choisir Villa" },
        "reveal_policy": "always"
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|mut ctx| {
            ctx.locale = "en-US".into();
            let surface = render_host_main(ctx).expect("host main");
            let sent = config_save::form_args(&surface);
            assert_eq!(sent["hint"], "Prefer 5 GHz");
            assert_eq!(sent["connection_steps"], "Choisir Villa");

            let saved = config_save::save(EMISSIONS, &surface, &stored, "en");
            assert_eq!(saved["hint"], stored["hint"]);
            assert_eq!(saved["connection_steps"]["fr"], "Choisir Villa");
        });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|mut ctx| {
            ctx.locale = "en-GB".into();
            let json = serde_json::to_string(&render_explore_detail(ctx).expect("detail")).unwrap();
            assert!(json.contains("Prefer 5 GHz"));
            assert!(!json.contains("5 GHz conseillé"));
        });
}
