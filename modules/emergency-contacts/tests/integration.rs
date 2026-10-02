//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use emergency_contacts::{render_explore_detail, render_home_card, render_host_main};
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

/// A host writing in English: the French label and note stay, and so do the note, the category
/// and the id the form does not carry; rows keep their place.
#[test]
#[serial]
fn a_save_in_english_keeps_the_french() {
    assert_eq!(
        config_save::localized_paths(EMISSIONS),
        ["contacts.label", "contacts.note"]
    );
    let stored = json!({
        "contacts": [
            { "id": "samu", "label": { "fr": "SAMU", "en": "Ambulance" }, "phone": "15",
              "note": { "fr": "Gratuit", "en": "Free" }, "category": "medical" },
            { "label": "", "phone": "" },
            { "id": "pompiers", "label": { "fr": "Pompiers" }, "phone": "18" }
        ],
        "host_visible_phone": "+33 6"
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|mut ctx| {
            ctx.locale = "en-US".into();
            let surface = render_host_main(ctx).expect("host main");
            let sent = config_save::form_args(&surface);
            // Stored order, the blank row where it was; ids on the filled rows only.
            assert_eq!(sent["contacts"][0]["id"], "samu");
            assert_eq!(sent["contacts"][0]["label"], "Ambulance");
            assert!(sent["contacts"][1].get("id").is_none());
            assert_eq!(sent["contacts"][2]["id"], "pompiers");
            assert_eq!(sent["contacts"].as_array().unwrap().len(), 6);

            let saved = config_save::save(EMISSIONS, &surface, &stored, "en");
            assert_eq!(saved["contacts"][0], stored["contacts"][0]);
            assert_eq!(saved["contacts"][2]["label"]["fr"], "Pompiers");
        });
}

/// Le numéro de l'hôte vient de son profil quand il ne l'a pas saisi ici.
///
/// Le champ du module existait parce que la plateforme ne portait pas le téléphone de l'hôte ;
/// elle le porte depuis `ctx.host`. Un hôte qui a rempli son compte n'a plus à le retaper, et le
/// §2.16 ne lui demande que ses contacts, sa pharmacie et son hôpital.
#[test]
#[serial]
fn the_host_row_falls_back_to_the_platform_profile() {
    let config = json!({ "contacts": [{ "label": "Pompiers", "phone": "18" }] });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .run(|mut ctx| {
            ctx.host = Some(portaki_sdk::context::HostProfile {
                name: "Claire".into(),
                phone: Some("+33612345678".into()),
                ..portaki_sdk::context::HostProfile::default()
            });
            let json = serde_json::to_string(&render_explore_detail(ctx).expect("detail"))
                .expect("surface json");
            assert!(json.contains("tel:+33612345678"), "{json}");
        });
}

/// Le numéro saisi ici gagne : c'est un choix délibéré, par exemple une ligne dédiée.
#[test]
#[serial]
fn a_number_the_host_typed_here_wins_over_the_profile() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|mut ctx| {
            ctx.host = Some(portaki_sdk::context::HostProfile {
                name: "Claire".into(),
                phone: Some("+33600000000".into()),
                ..portaki_sdk::context::HostProfile::default()
            });
            let json = serde_json::to_string(&render_explore_detail(ctx).expect("detail"))
                .expect("surface json");
            assert!(json.contains("+33 6 12 34 56 78"), "{json}");
            assert!(!json.contains("+33600000000"), "{json}");
        });
}

/// Sans contact ni numéro — ni ici, ni au profil — il n'y a rien à montrer.
#[test]
#[serial]
fn nothing_anywhere_still_shows_the_empty_state() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("detail");
            assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
        });
}

/// Le profil seul suffit à ouvrir la carte : un logement sans contact saisi montre quand même
/// qui appeler.
#[test]
#[serial]
fn the_profile_alone_is_enough_to_show_the_card() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .run(|mut ctx| {
            ctx.host = Some(portaki_sdk::context::HostProfile {
                name: "Claire".into(),
                phone: Some("+33612345678".into()),
                ..portaki_sdk::context::HostProfile::default()
            });
            let surface = render_explore_detail(ctx).expect("detail");
            assert!(!SurfaceAssertions::new(&surface).contains_type("EmptyState"));
            assert!(serde_json::to_string(&surface)
                .expect("json")
                .contains("tel:+33612345678"));
        });
}
