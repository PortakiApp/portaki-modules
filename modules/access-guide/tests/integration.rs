//! Integration-style unit tests with `portaki-test-utils`.

use chrono::{TimeZone, Utc};
use portaki_sdk::capability;
use serial_test::serial;

use access_guide::{
    map_markers, missing_code_tasks, on_config_updated, publish_readiness, render_explore_detail,
    render_home_card, render_host_main, render_host_stay, render_upcoming_card, ConfigUpdatedArgs,
    HostConfig, PrimaryMethod, StepRow,
};
use portaki_sdk::context::StayContext;
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::contracts::publish::PublishLevel;
use portaki_sdk::host::with_host;
use portaki_sdk::sdui::GeoPoint;
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
            // Legacy keybox config → keybox method label, and the icon that names it.
            assert!(json.contains("i18n:guest.method.keybox"));
            // The icon follows the method: a keybox is a key, never the generic car.
            assert!(json.contains("\"icon\":\"key\""));
            assert!(!json.contains("\"car\""));
            // Never leak secrets on the compact card.
            assert!(!json.contains("4821"));
            assert!(!json.contains("A17B"));
        });
}

/// The icon names the way in: a meeting is an hour, a reception desk is a handover. Same mapping
/// as the status cell — one method, one icon, wherever the method is announced.
#[test]
#[serial]
fn upcoming_card_icon_follows_the_method() {
    for (method, icon) in [
        ("keybox", "key"),
        ("door_code", "key"),
        ("smart_lock", "key"),
        ("in_person", "clock"),
        ("building_staff", "handshake"),
        ("host_greets", "user"),
        ("other", "key"),
    ] {
        let config = HostConfig {
            primary_method: method.into(),
            reveal_policy: "always".into(),
            // `other` with nothing else written is host silence, and silence draws an empty
            // state rather than a card: one field is enough to make the config real.
            address: "Ch. des Douaniers".into(),
            ..HostConfig::default()
        };
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&config)
            .run(|ctx| {
                let json =
                    serde_json::to_string(&render_upcoming_card(ctx).expect("surface")).unwrap();
                assert!(
                    json.contains(&format!("\"icon\":\"{icon}\"")),
                    "{method} should be announced with {icon} — {json}"
                );
            });
    }
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
            // Le moyen d'accès est le sous-titre de la carte, plus une rangée : la maquette
            // garde la carte au plan, aux codes et au chemin, et le reste est dans la sous-page.
            assert!(json.contains("\"subtitle\""));
            assert!(!json.contains("i18n:guest.method"));
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
            assert!(SurfaceAssertions::new(&surface).contains_type("Link"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("Se garer"));
            // Le rang à gauche, le type à droite : ce sont des emplacements du ListItem, pas des
            // enfants. En enfants, le badge se dessinait dans le corps de la ligne.
            assert!(
                json.contains(r#""leading":{"index":1}"#),
                "une étape porte son rang"
            );
            assert!(
                json.contains(r#""trailing":{"badge":"#),
                "le type d'étape est un badge de fin de ligne"
            );
        });
}

/// Only an `https` link reaches the guest: a plain text (what an untyped field held) is none.
#[test]
#[serial]
fn a_link_that_is_not_https_is_not_rendered() {
    let config = HostConfig {
        arrival_video_url: "Texte de test".into(),
        parking_map_url: "http://maps.example.com".into(),
        ..always_reveal_config()
    };
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .run(|ctx| {
            let json =
                serde_json::to_string(&render_explore_detail(ctx).expect("surface")).expect("json");
            assert!(!json.contains("Texte de test"), "{json}");
            assert!(!json.contains("http://maps.example.com"), "{json}");
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
            // Dans la rue : le tarif n'est dessiné que là.
            let mut config = always_reveal_config();
            config.parking_type = "street".into();
            // Une autre personne remet les clés : son nom et son numéro ne sont dessinés que là.
            config.handover_person = "other".into();
            let (mut ctx, host) = MockContext::host()
                .with_capabilities(&[capability::core::STORAGE])
                .with_config(&config)
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
            "building_floor",
            "building_note",
            "building_staff_desk_location",
            "building_staff_hours",
            "desk_after_hours",
            "global_note",
            "host_greets_contact_note",
            "host_greets_eta_hint",
            "in_person_meeting_place",
            "in_person_time_hint",
            "keybox_location",
            "late_arrival_note",
            "method_instructions",
            "parking_info",
            "parking_price",
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
        // The invocation's property (nil in the mock), never the one the payload names.
        assert_eq!(sent[0].property_id, Some(Uuid::nil()));
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

/// La carte d'une remise en main propre (§2.1).
///
/// <p>Elle n'avait qu'une tuile — l'heure d'arrivée, étirée sur toute la largeur — et ce que
/// l'hôte avait écrit du rendez-vous tombait en rangées grises sous l'adresse. La maquette donne
/// à chaque moyen d'accès une paire (libellé, valeur) à côté de l'arrivée, et pose le rendez-vous
/// en bandeau d'information.
#[test]
#[serial]
fn an_in_person_handover_reads_like_the_design() {
    let stored = json!({
        "primary_method": "in_person",
        "in_person_meeting_place": { "fr": "Devant le portail bleu" },
        "in_person_time_hint": { "fr": "16–19 h" },
        "in_person_contact": "+33 6 12 34 56 78",
        "in_person_meeting_lat": 43.70,
        "in_person_meeting_lng": 7.26,
        "address": "12 chemin de la Garoupe, Antibes"
    });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .with_coordinates(Some(GeoPoint::new(43.55, 7.12)))
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("card");
            let text = serde_json::to_string(&surface).expect("json");

            // Deux tuiles : le rendez-vous et l'arrivée. Plus jamais une case seule.
            assert!(text.contains("guest.inPerson.handover"), "{text}");
            assert!(text.contains("16–19 h"), "{text}");
            // Le rendez-vous en bandeau, pas en rangée grise.
            assert!(text.contains("InfoBanner"), "{text}");
            assert!(text.contains("Devant le portail bleu"), "{text}");
            // Les coordonnées décimales ne se lisent pas : le plan montre le point.
            assert!(!text.contains("guest.inPerson.coords"), "{text}");
        });
}

/// Deux lieux, deux repères — et le logement sur **ses** coordonnées.
///
/// <p>Le repère portait le nom du logement aux coordonnées du rendez-vous dès que l'hôte en
/// posait un : le livret annonçait le logement dans une rue où il n'est pas.
#[test]
#[serial]
fn the_map_puts_the_property_where_the_property_is() {
    let stored = json!({
        "primary_method": "in_person",
        "in_person_meeting_place": { "fr": "Gare" },
        "in_person_meeting_lat": 43.70,
        "in_person_meeting_lng": 7.26
    });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .with_coordinates(Some(GeoPoint::new(43.55, 7.12)))
        .run(|ctx| {
            let json = serde_json::to_value(render_home_card(ctx).expect("card")).unwrap();
            let map = find_map(&json).expect("un plan");
            let markers = map["markers"].as_array().expect("des repères");
            assert_eq!(markers.len(), 2, "{map}");
            let property = markers
                .iter()
                .find(|m| m["kind"] == "property")
                .expect("le repère du logement");
            assert_eq!(property["lat"], 43.55, "{map}");
            assert_eq!(property["lng"], 7.12, "{map}");
        });
}

fn find_map(node: &serde_json::Value) -> Option<&serde_json::Value> {
    if node.get("type").and_then(|t| t.as_str()) == Some("Map") {
        return Some(node);
    }
    match node {
        serde_json::Value::Object(map) => map.values().find_map(find_map),
        serde_json::Value::Array(items) => items.iter().find_map(find_map),
        _ => None,
    }
}

/// Les trois codes sortent en tuiles, dans l'ordre qu'on franchit : porte, immeuble, parking.
///
/// Le code du parking ne vivait qu'en rangée dans la sous-page, alors que les deux autres
/// étaient en tuile avec Copier sur la carte : devant une barrière fermée, on le cherchait.
#[test]
#[serial]
fn home_card_tiles_the_three_codes_in_the_order_one_crosses_them() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&HostConfig {
            parking_code: "P7788".into(),
            ..always_reveal_config()
        })
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("surface");
            let json = serde_json::to_string(&surface).expect("json");

            let door = json.find("4821").expect("le code de la boîte à clés");
            let building = json.find("A17B").expect("le digicode de l'immeuble");
            let parking = json.find("P7788").expect("le code de la barrière");
            assert!(door < building, "la porte avant l'immeuble");
            assert!(building < parking, "l'immeuble avant le parking");

            // En tuile, pas en rangée : le code du parking porte sa clé comme les autres.
            assert!(json.contains("i18n:guest.parking.code"));
        });
}

/// Un code reste un code : celui du parking se masque avec les autres tant que la révélation
/// n'a pas eu lieu, et ne s'offre pas à la copie.
#[test]
#[serial]
fn home_card_masks_the_parking_code_like_the_others() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&HostConfig {
            parking_code: "P7788".into(),
            reveal_policy: "day_before16h".into(),
            ..always_reveal_config()
        })
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("surface");
            let json = serde_json::to_string(&surface).expect("json");
            assert!(
                !json.contains("P7788"),
                "le code ne part pas avant son heure"
            );
            assert!(json.contains("••••••"));
        });
}

/// Les sept moyens d'accès rendent chacun leur carte, sans trou.
///
/// Cinq d'entre eux n'avaient aucun test de surface voyageur : un moyen sans code doit tout de
/// même porter sa tuile — le créneau de remise, les horaires de la réception, le nom de l'hôte —
/// sinon la carte s'ouvre sur une grille vide.
#[test]
#[serial]
fn every_access_method_draws_its_card() {
    for method in [
        "keybox",
        "door_code",
        "smart_lock",
        "in_person",
        "building_staff",
        "host_greets",
        "other",
    ] {
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&HostConfig {
                primary_method: method.into(),
                ..always_reveal_config()
            })
            .run(|ctx| {
                let surface = render_home_card(ctx).expect("surface");
                let json = serde_json::to_string(&surface).expect("json");
                assert!(
                    SurfaceAssertions::new(&surface).contains_type("KeyValue"),
                    "{method} : aucune tuile"
                );
                assert!(
                    !json.contains("\"children\":[]"),
                    "{method} : une grille vide"
                );
            });
    }
}

/// La consigne d'arrivée tardive, montrée à qui en a annoncé une (§2.1).
///
/// Trois conditions, et le silence de l'une suffit : l'hôte a écrit la consigne, le voyageur a
/// annoncé une heure en pré-arrivée, et cette heure est tardive. L'heure annoncée est le début du
/// créneau choisi — un plancher — donc « après 19 h » sur une entrée à 16 h est bien une arrivée
/// tardive, et c'est le seul créneau tardif que le formulaire propose.
fn late_arrival_card(note: Option<&str>, announced: Option<(u32, u32)>) -> String {
    late_arrival_surfaces(note, announced).0
}

/// La carte d'accueil et la sous-page : la consigne est en tête des deux.
fn late_arrival_surfaces(note: Option<&str>, announced: Option<(u32, u32)>) -> (String, String) {
    let (mut ctx, host) = MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&HostConfig {
            late_arrival_note: note.map(|text| I18nText::new(text, "")).unwrap_or_default(),
            ..always_reveal_config()
        })
        .build();
    ctx.timezone = "Europe/Paris".into();
    ctx.property.timezone = "Europe/Paris".into();
    ctx.stay = Some(StayContext {
        stay_id: Uuid::nil(),
        // 14 h UTC, 16 h à Paris : l'entrée du logement.
        checkin_at: Some(
            Utc.with_ymd_and_hms(2099, 7, 1, 14, 0, 0)
                .single()
                .expect("dt"),
        ),
        checkout_at: None,
        booking_channel: None,
        arrival_time_estimated: announced
            .map(|(h, m)| chrono::NaiveTime::from_hms_opt(h, m, 0).expect("heure")),
        ..StayContext::default()
    });
    with_host(host, ctx.clone(), || {
        (
            serde_json::to_string(&render_home_card(ctx.clone()).expect("carte")).expect("json"),
            serde_json::to_string(&render_explore_detail(ctx).expect("sous-page")).expect("json"),
        )
    })
}

#[test]
#[serial]
fn the_late_arrival_note_reaches_the_guest_who_announced_one() {
    // « Après 19 h » sur une entrée à 16 h : le dernier créneau du formulaire, donc une arrivée
    // tardive — sans cette borne la consigne serait morte dans presque tous les logements.
    let late = late_arrival_card(Some("Le coffre change de code après 22 h"), Some((19, 0)));
    assert!(
        late.contains("Le coffre change de code après 22 h"),
        "{late}"
    );
    assert!(late.contains("guest.lateArrival.title"), "{late}");

    // 23 h 30 saisi à la main, sans créneau : tardif aussi.
    let very_late = late_arrival_card(Some("Le coffre change de code après 22 h"), Some((23, 30)));
    assert!(
        very_late.contains("Le coffre change de code"),
        "{very_late}"
    );
}

#[test]
#[serial]
fn no_late_arrival_note_without_a_late_arrival() {
    let note = Some("Le coffre change de code après 22 h");

    // Une arrivée dans l'heure de l'entrée : la consigne ne le concerne pas.
    let on_time = late_arrival_card(note, Some((16, 0)));
    assert!(!on_time.contains("Le coffre change de code"), "{on_time}");
    assert!(!on_time.contains("guest.lateArrival.title"), "{on_time}");

    // Rien d'annoncé : rien à dire. Le module ne devine pas une heure d'arrivée.
    let silent = late_arrival_card(note, None);
    assert!(!silent.contains("Le coffre change de code"), "{silent}");

    // Et l'hôte qui n'a rien écrit ne laisse pas un bandeau vide (§0.5).
    let no_note = late_arrival_card(None, Some((23, 0)));
    assert!(!no_note.contains("guest.lateArrival.title"), "{no_note}");
}

fn parked(lat: f64, lng: f64) -> HostConfig {
    HostConfig {
        parking_enabled: true,
        parking_lat: Some(lat),
        parking_lng: Some(lng),
        ..HostConfig::default()
    }
}

/// L'épingle du parking part sur la Carte du livret, rangée au stationnement ; sans parking ou
/// sans épingle, rien.
#[test]
#[serial]
fn a_placed_parking_reaches_the_booklet_map() {
    let markers = |config: HostConfig| {
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&config)
            .run(|ctx| map_markers(ctx).expect("markers").markers)
    };
    let placed = markers(parked(43.58, 7.12));
    assert_eq!(placed.len(), 1);
    assert_eq!(placed[0].category.as_deref(), Some("parking"));
    assert_eq!(placed[0].label.as_deref(), Some("i18n:guest.parking"));
    assert!(markers(HostConfig {
        parking_enabled: false,
        ..parked(43.58, 7.12)
    })
    .is_empty());
    assert!(markers(HostConfig {
        parking_enabled: true,
        ..HostConfig::default()
    })
    .is_empty());
}

/// Au-delà de 2 km du logement, un avertissement qui ne bloque pas ; en deçà, ou sans logement
/// géocodé, rien.
#[test]
#[serial]
fn a_far_parking_pin_warns_without_blocking() {
    let home = GeoPoint {
        lat: 43.58,
        lng: 7.12,
    };
    let check = |config: HostConfig, home: Option<GeoPoint>| {
        MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .with_coordinates(home)
            .with_config(&config)
            .run(|ctx| {
                publish_readiness(ctx)
                    .expect("publishReadiness")
                    .items
                    .into_iter()
                    .find(|item| item.id == "config.parking_position")
            })
    };
    // 0,03° de latitude ≈ 3,3 km.
    let far = check(parked(43.61, 7.12), Some(home)).expect("warning");
    assert_eq!(far.level, PublishLevel::Recommended);
    assert!(!far.ok);
    assert_eq!(
        far.hint.get("fr"),
        "L'entrée du parking est à plus de 2 km du logement : vérifiez l'épingle."
    );
    // 0,005° ≈ 550 m.
    assert!(check(parked(43.585, 7.12), Some(home)).is_none());
    assert!(check(parked(43.61, 7.12), None).is_none());
}

// ── Remise des clés, réception, serrure, départ, séjour, À venir (spec Accès §2.4–2.6, §1) ──

fn detail_json(config: &HostConfig, stay: Option<StayContext>) -> String {
    let (mut ctx, host) = MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(config)
        .build();
    ctx.property.timezone = "Europe/Paris".into();
    ctx.stay = stay;
    with_host(host, ctx.clone(), || {
        serde_json::to_string(&render_explore_detail(ctx).expect("detail")).expect("json")
    })
}

fn stay_between(checkin: (i32, u32, u32), checkout: (i32, u32, u32)) -> Option<StayContext> {
    let at = |(y, m, d): (i32, u32, u32)| Utc.with_ymd_and_hms(y, m, d, 14, 0, 0).single();
    Some(StayContext {
        stay_id: Uuid::nil(),
        checkin_at: at(checkin),
        checkout_at: at(checkout),
        ..StayContext::default()
    })
}

#[test]
#[serial]
fn the_handover_slot_and_the_person_reach_the_booklet() {
    let config = HostConfig {
        primary_method: "in_person".into(),
        in_person_meeting_place: I18nText::new("Café du port", ""),
        in_person_time_hint: I18nText::new("en fin d'après-midi", ""),
        handover_slot_from: "16:00".into(),
        handover_slot_until: "19:00".into(),
        handover_person: "other".into(),
        handover_name: "Paulette".into(),
        handover_phone: "+33 6 12 34 56 78".into(),
        ..HostConfig::default()
    };
    let (ctx, host) = MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .build();
    let card = with_host(host, ctx.clone(), || {
        serde_json::to_string(&render_home_card(ctx).expect("card")).expect("json")
    });
    // Le créneau prend la place de l'indication libre dans la tuile.
    assert!(card.contains("16:00 – 19:00"));
    assert!(!card.contains("en fin d'après-midi"));
    let detail = detail_json(&config, None);
    assert!(detail.contains("Paulette"));
    assert!(detail.contains("tel:+33612345678"));
}

#[test]
#[serial]
fn the_reception_phone_and_after_hours_note_reach_the_booklet() {
    let config = HostConfig {
        primary_method: "building_staff".into(),
        building_staff_desk_location: I18nText::new("Hall", ""),
        desk_phone: "+33 4 93 00 00 00".into(),
        desk_after_hours: I18nText::new("Clés au coffre, code envoyé le jour même", ""),
        ..HostConfig::default()
    };
    let detail = detail_json(&config, None);
    assert!(detail.contains("tel:+33493000000"));
    assert!(detail.contains("i18n:guest.desk.call"));
    assert!(detail.contains("Clés au coffre, code envoyé le jour même"));
    // Une autre méthode : ni bouton ni consigne.
    let detail = detail_json(
        &HostConfig {
            primary_method: "keybox".into(),
            keybox_code: "4821".into(),
            ..config
        },
        None,
    );
    assert!(!detail.contains("tel:"));
    assert!(!detail.contains("Clés au coffre"));
}

#[test]
#[serial]
fn the_unlock_button_shows_in_the_stay_unless_the_host_follows_the_reveal() {
    const UNLOCK: &str = "i18n:guest.smartLock.unlock";
    // Codes visibles dès la réservation, arrivée en 2099 : le bouton attend l'arrivée, le code
    // de la serrure suit la révélation.
    let stay = stay_between((2099, 7, 1), (2099, 7, 5));
    let json = detail_json(&smart_lock_config(Some("nuki")), stay.clone());
    assert!(json.contains("9999"));
    assert!(!json.contains(UNLOCK));
    assert!(json.contains("getGuestCredential"));
    let json = detail_json(
        &HostConfig {
            unlock_window: "reveal".into(),
            ..smart_lock_config(Some("nuki"))
        },
        stay,
    );
    assert!(json.contains(UNLOCK));
    // Dans le séjour, le bouton est là.
    let json = detail_json(
        &smart_lock_config(Some("nuki")),
        stay_between((2020, 7, 1), (2099, 7, 5)),
    );
    assert!(json.contains(UNLOCK));
}

#[test]
#[serial]
fn after_check_out_the_card_is_the_address_alone() {
    let json = detail_json(
        &always_reveal_config(),
        stay_between((2020, 7, 1), (2020, 7, 5)),
    );
    assert!(json.contains("Ch. des Douaniers"));
    for gone in [
        "4821",
        "A17B",
        "••••••",
        "Se garer",
        "i18n:guest.reveal.lockedTitle",
    ] {
        assert!(!json.contains(gone), "{gone}");
    }
}

#[test]
#[serial]
fn a_smart_lock_without_a_backup_code_warns_without_blocking() {
    let readiness = |manual: &str| {
        MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&HostConfig {
                smart_lock_manual_code: manual.into(),
                ..smart_lock_config(Some("nuki"))
            })
            .run(|ctx| publish_readiness(ctx).expect("publishReadiness").items)
    };
    let items = readiness("");
    let backup = items
        .iter()
        .find(|item| item.id == "config.smart_lock_manual_code")
        .expect("avertissement");
    assert_eq!(backup.level, PublishLevel::Recommended);
    // La serrure liée émet les codes : l'entrée n'est pas bloquée.
    assert!(items.iter().any(|item| item.id == "entry-code" && item.ok));
    assert!(readiness("9999")
        .iter()
        .all(|item| item.id != "config.smart_lock_manual_code"));
}

#[test]
#[serial]
fn the_stay_card_says_when_the_code_is_missing() {
    let stay_card = |code: &str| {
        let (mut ctx, host) = MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&HostConfig {
                primary_method: "keybox".into(),
                keybox_code: code.into(),
                ..HostConfig::default()
            })
            .build();
        ctx.input = json!({ "stay": { "checkIn": "2099-08-24T14:00:00Z",
                                      "checkOut": "2099-08-29T08:00:00Z" } });
        with_host(host, ctx.clone(), || {
            serde_json::to_string(&render_host_stay(ctx).expect("stay")).expect("json")
        })
    };
    assert!(stay_card("").contains("i18n:host.stay.missing"));
    let shown = stay_card("4821");
    assert!(!shown.contains("i18n:host.stay.missing"));
    // L'encart dit quand, jamais le code.
    assert!(!shown.contains("4821"));
}

#[test]
fn a_missing_code_is_a_task_for_each_arrival_to_come() {
    use portaki_sdk::contracts::timeline::{TimelineStay, TimelineTasksArgs};
    let at = |raw: &str| {
        chrono::DateTime::parse_from_rfc3339(raw)
            .unwrap()
            .with_timezone(&Utc)
    };
    let stay = |name: &str, check_in: &str| TimelineStay {
        id: Uuid::new_v4(),
        check_in: at(check_in),
        check_out: at(check_in) + chrono::Duration::days(3),
        guest_name: name.into(),
        status: "UPCOMING".into(),
    };
    let args = TimelineTasksArgs {
        property_id: Uuid::nil(),
        from: at("2026-08-17T00:00:00Z"),
        to: at("2026-08-31T00:00:00Z"),
        stays: vec![
            stay("Ada", "2026-08-15T14:00:00Z"),   // déjà arrivée
            stay("Marie", "2026-08-21T14:00:00Z"), // dans 2 j
        ],
    };
    let now = at("2026-08-19T12:00:00Z");
    let missing = HostConfig {
        primary_method: "keybox".into(),
        ..HostConfig::default()
    }
    .to_model("fr");
    let tasks = missing_code_tasks(&missing, &args, now, "Europe/Paris");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].stay_id, Some(args.stays[1].id));
    assert_eq!(tasks[0].title.get("fr"), "Code manquant");
    assert_eq!(
        tasks[0].context.get("fr"),
        "Séjour de Marie · arrivée dans 2 j"
    );
    // La veille à 16 h, à Paris.
    assert_eq!(tasks[0].at, at("2026-08-20T14:00:00Z"));
    let set = HostConfig {
        primary_method: "keybox".into(),
        keybox_code: "4821".into(),
        ..HostConfig::default()
    }
    .to_model("fr");
    assert!(missing_code_tasks(&set, &args, now, "Europe/Paris").is_empty());
}
