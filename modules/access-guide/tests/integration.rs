//! Integration-style unit tests with `portaki-test-utils`.

use chrono::{TimeZone, Utc};
use portaki_sdk::capability;
use serial_test::serial;

use access_guide::{
    publish_readiness, render_explore_detail, render_home_card, render_host_main,
    render_upcoming_card, HostConfig, PrimaryMethod, RevealPolicy, StepRow, StepTextRow,
};
use portaki_sdk::context::StayContext;
use portaki_sdk::host::with_host;
use portaki_test_utils::{MockContext, SurfaceAssertions};
use serde_json::json;
use uuid::Uuid;

fn sample_config_bytes() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "address": "Ch. des Douaniers",
        "gate_code": "A17B",
        "keybox_code": "4821",
        "parking_info": "Résident · rue Aubernon",
        "parking_map_url": "https://maps.example.com",
        "arrival_video_url": "https://video.example.com",
        // Legacy bilingual step titles still accepted (prefer fr, then en → texts/fr + texts/en).
        "global_note": "Sonnette à gauche",
        "steps_json": r#"[{"id":"1","kind":"parking","title":{"fr":"Se garer","en":"Park"},"detail":{"fr":"Place résident","en":"Resident spot"}}]"#
    }))
    .expect("config json")
}

// The form is checked across methods (one surface shows one method): the key sets, not the
// single-surface assertion.
#[allow(dead_code)]
#[path = "../../../support/config_form.rs"]
mod config_form;

fn always_reveal_config() -> HostConfig {
    HostConfig {
        primary_method: "keybox".into(),
        keybox_location: "À droite de la porte".into(),
        keybox_code: "4821".into(),
        building_access_enabled: true,
        building_access_gate_code: "A17B".into(),
        parking_enabled: true,
        parking_map_url: "https://maps.example.com".into(),
        parking_info_fr: "Rue A".into(),
        address: "Ch. des Douaniers".into(),
        arrival_video_url: "https://video.example.com".into(),
        reveal_policy: "always".into(),
        global_note_fr: "Sonnette à gauche".into(),
        steps: vec![StepRow {
            kind: Some("parking".into()),
        }],
        steps_fr: vec![StepTextRow {
            title: "Se garer".into(),
            detail: "Place résident".into(),
        }],
        ..HostConfig::default()
    }
}

fn smart_lock_config(provider: Option<&str>) -> HostConfig {
    HostConfig {
        primary_method: "smart_lock".into(),
        smart_lock_manual_code: "9999".into(),
        smart_lock_provider_module_id: provider.unwrap_or_default().into(),
        address: "1 rue Test".into(),
        reveal_policy: "always".into(),
        method_instructions_fr: "Appuyer sur unlock".into(),
        ..HostConfig::default()
    }
}

#[test]
#[serial]
fn home_card_empty_without_config() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            assert!(
                SurfaceAssertions::new(&render_home_card(ctx).expect("surface"))
                    .contains_type("EmptyState")
            );
        });
}

#[test]
#[serial]
fn upcoming_card_renders_compact_method_summary() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_upcoming_card(ctx).expect("surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            // Compact card must not embed the full access glance (map / codes).
            assert!(!SurfaceAssertions::new(&surface).contains_type("Map"));
            assert!(!SurfaceAssertions::new(&surface).contains_type("KeyValue"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("upcoming.card"));
            assert!(json.contains("i18n:nav.access-guide"));
            // Legacy keybox config → keybox method label.
            assert!(json.contains("i18n:guest.method.keybox"));
            // Never leak secrets on the compact card.
            assert!(!json.contains("4821"));
            assert!(!json.contains("A17B"));
        });
}

#[test]
#[serial]
fn upcoming_card_empty_without_config() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            assert!(
                SurfaceAssertions::new(&render_upcoming_card(ctx).expect("surface"))
                    .contains_type("EmptyState")
            );
        });
}

#[test]
#[serial]
fn home_card_masks_secrets_without_stay() {
    // Legacy config defaults to day_before_16h; no checkin → fail-safe lock.
    // Before the platform holds the config: the flat legacy blob, texts embedded.
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", sample_config_bytes())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("KeyValue"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Button"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Map"));
            assert!(SurfaceAssertions::new(&surface).contains_type("InfoBanner"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("openOverlay"));
            assert!(json.contains("explore.detail"));
            assert!(json.contains("fullscreen"));
            assert!(
                json.contains("i18n:nav.access-guide"),
                "card title must use access-guide nav key, not another module's home.card.title"
            );
            assert!(
                !json.contains("i18n:nav.appliances") && !json.contains("i18n:home.card.title"),
                "must not emit appliances / colliding home.card.title key"
            );
            assert!(json.contains("i18n:guest.method"));
            assert!(json.contains("i18n:guest.openMaps"));
            assert!(!json.contains("4821"));
            assert!(!json.contains("A17B"));
            assert!(json.contains("••••••"));
            assert!(json.contains("\"mono\":true") || json.contains("\"mono\": true"));
        });
}

#[test]
#[serial]
fn home_card_emits_keybox_location_i18n_when_configured() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&always_reveal_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("surface");
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("i18n:nav.access-guide"));
            assert!(json.contains("i18n:guest.keybox.location"));
            assert!(json.contains("i18n:guest.keybox.code"));
            assert!(json.contains("À droite de la porte"));
        });
}

#[test]
#[serial]
fn home_card_reveals_secrets_when_policy_always() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&always_reveal_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("surface");
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("4821"));
            assert!(json.contains("A17B"));
            assert!(!json.contains("••••••"));
        });
}

#[test]
#[serial]
fn detail_has_steps_and_video() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&always_reveal_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Badge"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Link"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("Se garer"));
        });
}

#[test]
#[serial]
fn smart_lock_provider_emits_unlock_commands_when_revealed() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&smart_lock_config(Some("nuki")))
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("surface");
            let json = serde_json::to_string(&surface).expect("json");
            assert!(
                json.contains("\"type\":\"command\"") || json.contains("\"type\": \"command\"")
            );
            assert!(json.contains("nuki"));
            assert!(json.contains("unlock"));
            assert!(json.contains("getGuestCredential"));
            assert!(json.contains("9999"));
        });
}

#[test]
#[serial]
fn smart_lock_without_provider_shows_manual_fallback_only() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&smart_lock_config(None))
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("surface");
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("9999"));
            assert!(json.contains("Appuyer sur unlock"));
            assert!(!json.contains("getGuestCredential"));
        });
}

#[test]
#[serial]
fn smart_lock_provider_hides_cta_when_not_revealed() {
    let (mut ctx, host) = MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&HostConfig {
            reveal_policy: "at_checkin".into(),
            ..smart_lock_config(Some("nuki"))
        })
        .build();
    ctx.timezone = "Europe/Paris".into();
    ctx.property.timezone = "Europe/Paris".into();
    ctx.stay = Some(StayContext {
        stay_id: Uuid::nil(),
        checkin_at: Some(
            Utc.with_ymd_and_hms(2099, 1, 1, 15, 0, 0)
                .single()
                .expect("dt"),
        ),
        checkout_at: None,
        booking_channel: None,
        ..StayContext::default()
    });

    with_host(host, ctx.clone(), || {
        let surface = render_home_card(ctx).expect("surface");
        let json = serde_json::to_string(&surface).expect("json");
        assert!(!json.contains("unlock"));
        assert!(!json.contains("getGuestCredential"));
        assert!(!json.contains("9999"));
        assert!(json.contains("••••••"));
    });
}

#[test]
#[serial]
fn host_main_hides_reveal_for_no_code_methods_without_layers() {
    for method in [
        PrimaryMethod::InPerson,
        PrimaryMethod::BuildingStaff,
        PrimaryMethod::HostGreets,
        PrimaryMethod::Other,
    ] {
        MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .run(|ctx| {
                let mut ctx = ctx;
                ctx.input = json!({
                    "primary_method": method.as_wire(),
                    "building_access_enabled": false,
                    "parking_enabled": false,
                });
                let surface = render_host_main(ctx).expect("host main");
                let json = serde_json::to_string(&surface).expect("json");
                assert!(
                    !json.contains("i18n:host.section.reveal"),
                    "reveal section must stay hidden for {method:?}"
                );
            });
    }
}

#[test]
#[serial]
fn host_main_shows_reveal_for_code_methods() {
    for method in [
        PrimaryMethod::Keybox,
        PrimaryMethod::DoorCode,
        PrimaryMethod::SmartLock,
    ] {
        MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .run(|ctx| {
                let mut ctx = ctx;
                ctx.input = json!({
                    "primary_method": method.as_wire(),
                    "building_access_enabled": false,
                    "parking_enabled": false,
                });
                let surface = render_host_main(ctx).expect("host main");
                let json = serde_json::to_string(&surface).expect("json");
                assert!(
                    json.contains("i18n:host.section.reveal"),
                    "reveal section must show for {method:?}"
                );
            });
    }
}

#[test]
#[serial]
fn host_main_shows_reveal_for_in_person_when_building_layer_enabled() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let mut ctx = ctx;
            ctx.input = json!({
                "primary_method": PrimaryMethod::InPerson.as_wire(),
                "building_access_enabled": true,
                "parking_enabled": false,
            });
            let surface = render_host_main(ctx).expect("host main");
            let json = serde_json::to_string(&surface).expect("json");
            assert!(
                json.contains("i18n:host.section.reveal"),
                "building digicode still needs reveal timing"
            );
        });
}

/// Every method, both layers on, both languages: the union of what the form sends.
fn host_forms() -> Vec<(String, portaki_sdk::sdui::surface::Surface)> {
    let mut surfaces = Vec::new();
    for locale in ["fr-FR", "en-US"] {
        for method in PrimaryMethod::ALL {
            let (mut ctx, host) = MockContext::host()
                .with_capabilities(&[capability::core::STORAGE])
                .with_config(&always_reveal_config())
                .build();
            ctx.locale = locale.into();
            ctx.input = json!({
                "primary_method": method.as_wire(),
                "building_access_enabled": true,
                "parking_enabled": true,
            });
            let surface =
                with_host(host, ctx.clone(), || render_host_main(ctx)).expect("host main");
            surfaces.push((format!("{locale} {method:?}"), surface));
        }
    }
    surfaces
}

/// `config_form::form_keys` reads `name`; the map picker names its three inputs apart.
fn picker_keys(surface: &portaki_sdk::sdui::surface::Surface) -> Vec<String> {
    fn walk(value: &serde_json::Value, out: &mut Vec<String>) {
        match value {
            serde_json::Value::Object(object) => {
                for key in ["addressName", "latName", "lngName"] {
                    if let Some(name) = object.get(key).and_then(|v| v.as_str()) {
                        out.push(name.to_string());
                    }
                }
                object.values().for_each(|v| walk(v, out));
            }
            serde_json::Value::Array(items) => items.iter().for_each(|v| walk(v, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk(&serde_json::to_value(surface).unwrap(), &mut out);
    out
}

#[test]
#[serial]
fn the_host_form_sends_the_declared_keys() {
    let declared = config_form::declared_keys(concat!(env!("OUT_DIR"), "/portaki-emissions"));
    let mut sent = std::collections::BTreeSet::new();
    for (case, surface) in host_forms() {
        let mut keys = config_form::form_keys(&surface);
        keys.extend(picker_keys(&surface));
        let unknown: Vec<_> = keys.difference(&declared).collect();
        assert!(
            unknown.is_empty(),
            "{case}: the platform would refuse {unknown:?}"
        );
        sent.extend(keys);
    }
    let missing: Vec<_> = declared.difference(&sent).collect();
    assert!(missing.is_empty(), "no form fills {missing:?}");
}

#[test]
#[serial]
fn codes_are_never_sent_back_to_the_form() {
    let codes = HostConfig {
        door_code: "D00R".into(),
        smart_lock_manual_code: "SM4RT".into(),
        parking_code: "P4RK".into(),
        ..always_reveal_config()
    };
    for method in [
        PrimaryMethod::Keybox,
        PrimaryMethod::DoorCode,
        PrimaryMethod::SmartLock,
    ] {
        MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&codes)
            .run(|ctx| {
                let mut ctx = ctx;
                ctx.input = json!({ "primary_method": method.as_wire() });
                let json = serde_json::to_string(&render_host_main(ctx).expect("host main"))
                    .expect("json");
                for code in ["4821", "A17B", "D00R", "SM4RT", "P4RK"] {
                    assert!(!json.contains(code), "{method:?} sends {code} back");
                }
                assert!(json.contains("i18n:host.secret.keep"));
                assert!(json.contains("À droite de la porte") || method != PrimaryMethod::Keybox);
            });
    }
}

#[test]
#[serial]
fn the_host_edits_the_copy_of_its_own_language() {
    let config = HostConfig {
        global_note_en: "Ring twice".into(),
        ..always_reveal_config()
    };
    let (mut ctx, host) = MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .build();
    ctx.locale = "en-US".into();
    let json = with_host(host, ctx.clone(), || {
        serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json")
    });
    assert!(json.contains("global_note_en") && json.contains("Ring twice"));
    assert!(!json.contains("global_note_fr") && !json.contains("Sonnette à gauche"));
}

/// Before the platform held it, the KV kept a flat blob; the import takes only the keys the
/// form still uses (`address`, `keybox_code`…). The gate code, the copy and the steps are still
/// read from the KV.
#[test]
#[serial]
fn a_flat_legacy_blob_survives_the_import() {
    let imported = json!({
        "address": "Ch. des Douaniers",
        "keybox_code": "4821",
        "parking_map_url": "https://maps.example.com",
        "arrival_video_url": "https://video.example.com"
    });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_kv("config", sample_config_bytes())
        .with_config(&imported)
        .run(|ctx| {
            let config = HostConfig::read(&ctx).expect("config");
            assert_eq!(config.method(), Some(PrimaryMethod::Keybox));
            assert_eq!(config.keybox_code, "4821");
            assert!(config.building_access_enabled);
            assert_eq!(config.building_access_gate_code, "A17B");
            assert_eq!(config.global_note_fr, "Sonnette à gauche");
            assert_eq!(config.parking_info_fr, "Résident · rue Aubernon");
            assert_eq!(config.steps_fr[0].title, "Se garer");
            assert_eq!(config.steps_en[0].title, "Park");
            assert_eq!(config.reveal(), RevealPolicy::DayBefore16h);
            let texts = config.guest_texts("en-US", "fr-FR");
            assert_eq!(texts.steps[0].title, "Park");
        });
}

/// The redesigned blob nested the method and kept the copy in `texts/{lang}`: both still read,
/// the pre-rename policy included — until the platform holds the key, even empty.
#[test]
#[serial]
fn a_nested_blob_and_its_texts_survive_the_import() {
    let blob = json!({
        "primary_method": "keybox",
        "method": { "kind": "keybox", "location": "Sous le pot", "code": "4821" },
        "arrival": { "address": "Rue X", "steps": [{ "id": "a", "kind": "door" }] },
        "reveal_policy": "hours_before24"
    });
    let texts = |note: &str| {
        serde_json::to_vec(&json!({
            "global_note": note,
            "steps": [{ "id": "a", "title": note }]
        }))
        .unwrap()
    };
    let run = |held: serde_json::Value, check: &dyn Fn(HostConfig)| {
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_kv("config", serde_json::to_vec(&blob).unwrap())
            .with_kv("texts/fr", texts("Note FR"))
            .with_kv("texts/en", texts("Note EN"))
            .with_config(&held)
            .run(|ctx| check(HostConfig::read(&ctx).expect("config")));
    };
    run(json!({ "primary_method": "keybox" }), &|config| {
        assert_eq!(config.keybox_location, "Sous le pot");
        assert_eq!(config.keybox_code, "4821");
        assert_eq!(config.address, "Rue X");
        assert_eq!(config.reveal(), RevealPolicy::HoursBefore24);
        assert_eq!(config.texts("fr").global_note, "Note FR");
        assert_eq!(config.texts("en").global_note, "Note EN");
        assert_eq!(config.texts("en").steps[0].title, "Note EN");
        assert_eq!(
            config.to_model().parse_steps()[0].kind.as_deref(),
            Some("door")
        );
    });
    run(
        json!({ "primary_method": "keybox", "global_note_fr": "", "reveal_policy": "always" }),
        &|config| {
            assert_eq!(config.texts("fr").global_note, "");
            assert_eq!(config.texts("en").global_note, "Note EN");
            assert_eq!(config.reveal(), RevealPolicy::Always);
        },
    );
}

#[test]
#[serial]
fn publish_readiness_requires_code_for_code_methods() {
    for (code, ok) in [("", false), ("4821", true)] {
        MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&HostConfig {
                primary_method: "keybox".into(),
                keybox_location: "Porte".into(),
                keybox_code: code.into(),
                ..HostConfig::default()
            })
            .run(|ctx| {
                let items = publish_readiness(ctx).expect("publishReadiness").items;
                assert_eq!(items.len(), 1);
                assert_eq!(items[0].id, "entry-code");
                assert_eq!(items[0].ok, ok);
            });
    }
}

/// No method yet: the declared `primary_method` (required) blocks, not this check.
#[test]
#[serial]
fn publish_readiness_empty_without_code_method() {
    for config in [
        HostConfig::default(),
        HostConfig {
            primary_method: "in_person".into(),
            in_person_meeting_place: "Gare".into(),
            ..HostConfig::default()
        },
    ] {
        MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&config)
            .run(|ctx| {
                assert!(publish_readiness(ctx)
                    .expect("publishReadiness")
                    .items
                    .is_empty());
            });
    }
}

/// Not geocoded: no property map and no Maps link, never a default position.
#[test]
#[serial]
fn no_map_without_coordinates() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_coordinates(None)
        .with_config(&HostConfig {
            parking_map_url: String::new(),
            ..always_reveal_config()
        })
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("surface");
            assert!(!SurfaceAssertions::new(&surface).contains_type("Map"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(!json.contains("i18n:guest.openMaps"));
            assert!(json.contains("À droite de la porte"));
        });
}
