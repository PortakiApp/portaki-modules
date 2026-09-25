//! Integration-style unit tests with `portaki-test-utils`.

use chrono::{TimeZone, Utc};
use portaki_sdk::capability;
use serial_test::serial;

use access_guide::{
    on_config_updated, publish_readiness, render_explore_detail, render_home_card,
    render_host_main, render_upcoming_card, ConfigUpdatedArgs, HostConfig, PrimaryMethod, StepRow,
};
use portaki_sdk::context::StayContext;
use portaki_sdk::contracts::i18n::I18nText;
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
#[path = "../../../support/config_save.rs"]
mod config_save;

const EMISSIONS: &str = concat!(env!("OUT_DIR"), "/portaki-emissions");

fn always_reveal_config() -> HostConfig {
    HostConfig {
        primary_method: "keybox".into(),
        keybox_location: I18nText::new("À droite de la porte", ""),
        keybox_code: "4821".into(),
        building_access_enabled: true,
        building_access_gate_code: "A17B".into(),
        parking_enabled: true,
        parking_map_url: "https://maps.example.com".into(),
        parking_info: I18nText::new("Rue A", ""),
        address: "Ch. des Douaniers".into(),
        arrival_video_url: "https://video.example.com".into(),
        reveal_policy: "always".into(),
        global_note: I18nText::new("Sonnette à gauche", ""),
        steps: vec![StepRow {
            id: "park".into(),
            kind: Some("parking".into()),
            title: I18nText::new("Se garer", ""),
            detail: I18nText::new("Place résident", ""),
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
        method_instructions: I18nText::new("Appuyer sur unlock", ""),
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
    let declared = config_form::declared_keys(EMISSIONS);
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
        global_note: I18nText::new("Sonnette à gauche", "Ring twice"),
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
    assert!(json.contains("Ring twice"));
    assert!(!json.contains("Sonnette à gauche"));
}

/// A host writing in English: every French text stays — method, layers, steps — and so do the
/// codes the form never sends back; the steps keep their place and their id, the blank one where
/// it was.
#[test]
#[serial]
fn a_save_in_english_keeps_the_french() {
    assert_eq!(
        config_save::localized_paths(EMISSIONS),
        [
            "building_access_intercom",
            "building_note",
            "building_staff_desk_location",
            "building_staff_hours",
            "global_note",
            "host_greets_contact_note",
            "host_greets_eta_hint",
            "in_person_meeting_place",
            "in_person_time_hint",
            "keybox_location",
            "method_instructions",
            "parking_info",
            "steps.detail",
            "steps.title",
        ]
    );
    let stored = json!({
        "primary_method": "keybox",
        "keybox_location": { "fr": "Sous le pot", "en": "Under the pot" },
        "keybox_code": "4821",
        "method_instructions": { "fr": "Tourner", "en": "Turn" },
        "building_access_enabled": true,
        "building_access_intercom": { "fr": "Apt 3" },
        "building_note": { "fr": "Portail vert", "en": "Green gate" },
        "parking_enabled": true,
        "parking_info": { "fr": "Place 8", "en": "Spot 8" },
        "global_note": { "fr": "Bienvenue", "en": "Welcome" },
        "reveal_policy": "always",
        "steps": [
            { "id": "a", "kind": "parking", "title": { "fr": "Se garer", "en": "Park" },
              "detail": { "fr": "Rampe à gauche", "en": "Ramp on the left" } },
            { "kind": "" },
            { "id": "b", "kind": "door", "title": { "fr": "Monter" }, "detail": { "fr": "2e étage" } }
        ]
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|mut ctx| {
            ctx.locale = "en-US".into();
            let surface = render_host_main(ctx).expect("host main");
            let sent = config_save::form_args(&surface);
            // Stored order, the blank row where it was; ids on the filled rows only.
            assert_eq!(sent["steps"].as_array().unwrap().len(), 3);
            assert_eq!(sent["steps"][0]["id"], "a");
            assert_eq!(sent["steps"][0]["title"], "Park");
            assert!(sent["steps"][1].get("id").is_none());
            assert_eq!(sent["steps"][2]["id"], "b");
            assert_eq!(sent["global_note"], "Welcome");

            let saved = config_save::save(EMISSIONS, &surface, &stored, "en");
            for key in [
                "keybox_location",
                "keybox_code",
                "method_instructions",
                "building_note",
                "parking_info",
                "global_note",
            ] {
                assert_eq!(saved[key], stored[key], "{key}");
            }
            assert_eq!(saved["building_access_intercom"]["fr"], "Apt 3");
            assert_eq!(saved["steps"][0], stored["steps"][0]);
            assert_eq!(saved["steps"][2]["id"], "b");
            assert_eq!(saved["steps"][2]["title"]["fr"], "Monter");
            assert_eq!(saved["steps"][2]["detail"]["fr"], "2e étage");
        });
}

/// A step the host removes: the step list blanks all of `steps.N.*`, its id too — the platform
/// replaces the row rather than merging into it, and the guest no longer shows it.
#[test]
#[serial]
fn a_removed_step_is_cleared() {
    let stored = json!({
        "steps": [
            { "id": "a", "kind": "parking", "title": { "fr": "Se garer", "en": "Park" } },
            { "id": "b", "kind": "door", "title": { "fr": "Monter", "en": "Go up" } }
        ]
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|ctx| {
            let mut surface = render_host_main(ctx).expect("host main");
            // What the dashboard's StepList does on removal.
            let mut value = serde_json::to_value(&surface).unwrap();
            blank_names_under(&mut value, "steps.0.");
            surface = serde_json::from_value(value).unwrap();
            let saved = config_save::save(EMISSIONS, &surface, &stored, "fr");
            assert_eq!(
                saved["steps"][0],
                json!({ "id": "", "kind": "", "title": "", "detail": "" })
            );
            assert_eq!(saved["steps"][1]["id"], "b");
            assert_eq!(saved["steps"][1]["title"], stored["steps"][1]["title"]);
            let config: HostConfig = serde_json::from_value(saved).unwrap();
            let titles: Vec<String> = config
                .texts("fr")
                .steps
                .into_iter()
                .map(|s| s.title)
                .collect();
            assert_eq!(titles, ["Monter"]);
        });
}

fn blank_names_under(value: &mut serde_json::Value, prefix: &str) {
    match value {
        serde_json::Value::Object(object) => {
            let named = object
                .get("name")
                .and_then(|n| n.as_str())
                .is_some_and(|n| n.starts_with(prefix));
            if named && object.contains_key("value") {
                object.insert("value".into(), json!(""));
            }
            object
                .values_mut()
                .for_each(|v| blank_names_under(v, prefix));
        }
        serde_json::Value::Array(items) => {
            items.iter_mut().for_each(|v| blank_names_under(v, prefix))
        }
        _ => {}
    }
}

/// The coordinates are numbers (the platform reads « 43,7 » and a blank for them), and the saved
/// point goes back on the map so a save does not wipe it.
#[test]
#[serial]
fn coordinates_are_numbers_and_stay_on_the_map() {
    let fields = config_save::declared_fields(EMISSIONS);
    for key in [
        "in_person_meeting_lat",
        "in_person_meeting_lng",
        "arrival_lat",
        "arrival_lng",
    ] {
        let field = fields.iter().find(|f| f["key"] == key).expect(key);
        assert_eq!(field["type"], "number", "{key}");
    }
    let stored = json!({
        "primary_method": "in_person",
        "in_person_meeting_place": { "fr": "Gare" },
        "in_person_meeting_lat": 43.7,
        "in_person_meeting_lng": 7.26,
        "address": "Rue X",
        "arrival_lat": 43.5,
        "arrival_lng": 7.1
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|ctx| {
            let json = serde_json::to_value(render_host_main(ctx).expect("host main")).unwrap();
            let text = json.to_string();
            for point in ["43.7", "7.26", "43.5", "7.1"] {
                assert!(text.contains(point), "{point} is not on the map");
            }
        });
}

#[test]
#[serial]
fn publish_readiness_requires_code_for_code_methods() {
    for (code, ok) in [("", false), ("4821", true)] {
        MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&HostConfig {
                primary_method: "keybox".into(),
                keybox_location: I18nText::new("Porte", ""),
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
            in_person_meeting_place: I18nText::new("Gare", ""),
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

/// `onConfigUpdated`: the payload the platform sends after a save (PR platform#622), with the
/// config as saved.
fn config_updated(
    saved: &serde_json::Value,
    payload: serde_json::Value,
) -> Vec<portaki_sdk::host::email::SendEmailArgs> {
    let args: ConfigUpdatedArgs = serde_json::from_value(payload).expect("payload");
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(saved)
        .run_with(|ctx, host| {
            on_config_updated(ctx, args).expect("onConfigUpdated");
            host.sent_emails()
        })
}

fn changed(keys: &[&str]) -> serde_json::Value {
    json!({ "propertyId": Uuid::nil().to_string(), "changedKeys": keys })
}

/// Every code set; the method and the layers say which ones the guest uses.
fn saved(method: &str, building: bool, parking: bool) -> serde_json::Value {
    json!({
        "primary_method": method,
        "keybox_code": "4821",
        "door_code": "1234",
        "smart_lock_manual_code": "9999",
        "building_access_enabled": building,
        "building_access_gate_code": "A17B",
        "parking_enabled": parking,
        "parking_code": "P1"
    })
}

#[test]
#[serial]
fn a_new_active_code_emails_the_guests() {
    let property = Uuid::new_v4();
    for (config, key) in [
        (saved("keybox", false, false), "keybox_code"),
        (saved("door_code", false, false), "door_code"),
        (saved("smart_lock", false, false), "smart_lock_manual_code"),
        (saved("in_person", true, false), "building_access_gate_code"),
        (saved("other", false, true), "parking_code"),
    ] {
        let sent = config_updated(
            &config,
            json!({ "propertyId": property.to_string(), "changedKeys": ["global_note", key] }),
        );
        assert_eq!(sent.len(), 1, "{key}");
        assert_eq!(sent[0].email_id, "code-changed");
        assert_eq!(
            sent[0].audience,
            portaki_sdk::host::email::EmailAudience::PropertyEligibleGuests
        );
        assert_eq!(sent[0].property_id, Some(property));
        assert!(!sent[0].content.subject.fr.is_empty());
    }
}

#[test]
#[serial]
fn the_code_of_an_inactive_method_or_layer_sends_nothing() {
    for (config, key) in [
        (saved("door_code", false, false), "keybox_code"),
        (saved("keybox", false, false), "door_code"),
        (saved("keybox", false, false), "smart_lock_manual_code"),
        (saved("keybox", false, false), "building_access_gate_code"),
        (saved("keybox", false, false), "parking_code"),
    ] {
        assert!(config_updated(&config, changed(&[key])).is_empty(), "{key}");
    }
}

/// Switching the method or a layer changes the code the guest uses — when one is set.
#[test]
#[serial]
fn a_switch_to_a_set_code_emails_the_guests() {
    for key in [
        "primary_method",
        "building_access_enabled",
        "parking_enabled",
    ] {
        assert_eq!(
            config_updated(&saved("door_code", false, false), changed(&[key])).len(),
            1,
            "{key}"
        );
        assert_eq!(
            config_updated(&saved("in_person", false, true), changed(&[key])).len(),
            1,
            "{key}"
        );
        // Nothing active is set: nothing for the guest to use.
        let no_code = json!({ "primary_method": "keybox", "building_access_enabled": true });
        assert!(
            config_updated(&no_code, changed(&[key])).is_empty(),
            "{key}"
        );
        assert!(
            config_updated(&saved("host_greets", false, false), changed(&[key])).is_empty(),
            "{key}"
        );
    }
}

#[test]
#[serial]
fn no_email_without_a_code_change() {
    let config = saved("keybox", true, true);
    for payload in [
        changed(&["global_note", "steps", "keybox_location"]),
        changed(&[]),
        json!({}),
    ] {
        assert!(
            config_updated(&config, payload.clone()).is_empty(),
            "{payload}"
        );
    }
}
