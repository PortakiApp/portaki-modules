//! Integration-style unit tests with `portaki-test-utils`.

use ev_parking::{
    email_context, render_explore_detail, render_home_card, render_host_main, EmailContextArgs,
};
use portaki_sdk::capability;
use portaki_sdk::prelude::EmailTemplateKey;
use serial_test::serial;

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
        "spot_label": "P2 / Place 14",
        "charger_pin": "4821",
        "parking_code": "1234",
        "map_url": "https://maps.example/parking",
        "instructions": "Left entrance after the barrier",
        "reveal_policy": "day_before_16h"
    })
}

fn always_reveal_config() -> Value {
    json!({
        "spot_label": "P2 / Place 14",
        "charger_pin": "4821",
        "parking_code": "1234",
        "reveal_policy": "always"
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
fn home_card_renders_with_config_and_masks_secrets() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("home card");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("KeyValue"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Link"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("P2 / Place 14"));
            assert!(json.contains("i18n:nav.ev-parking"));
            assert!(json.contains("https://maps.example/parking"));
            assert!(!json.contains("4821"));
            assert!(!json.contains("1234"));
            assert!(json.contains("••••••"));
        });
}

#[test]
#[serial]
fn the_tiles_carry_their_own_copy_when_revealed() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&always_reveal_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("detail");
            let json = serde_json::to_string(&surface).expect("json");

            // La copie vit dans la tuile et non dans un bouton en dessous : « valeur + Copier » est
            // une seule chose pour le voyageur, et le livret rend l'affordance lui-même.
            assert!(json.contains("\"layout\":\"tile\""));
            assert!(json.contains("\"copy\":true"));
            assert!(json.contains("guest.copyParkingCode"));
            assert!(json.contains("guest.copyChargerPin"));
            assert!(json.contains("4821"));
            assert!(json.contains("1234"));
        });
}

/// Une valeur masquée ne se copie pas : il n'y aurait que des points dans le presse-papiers, et le
/// voyageur croirait tenir son code.
#[test]
#[serial]
fn a_masked_tile_offers_no_copy() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("detail");
            let json = serde_json::to_string(&surface).expect("json");

            assert!(json.contains("\"layout\":\"tile\""));
            assert!(!json.contains("\"copy\":true"));
        });
}

#[test]
#[serial]
fn the_host_form_sends_the_declared_keys() {
    // Payante et sur réservation : le prix et la consigne ne sont dessinés que là.
    let mut config = sample_config();
    config["pricing"] = json!("per_kwh");
    config["price"] = json!("0,25 €/kWh");
    config["booking_required"] = json!(true);
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            config_form::assert_form_matches_config(
                concat!(env!("OUT_DIR"), "/portaki-emissions"),
                &surface,
                &[],
            );
            // The codes are never sent back to the form: blank keeps them.
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("P2 / Place 14"));
            assert!(!json.contains("4821"));
            assert!(!json.contains("1234"));
        });
}

#[test]
#[serial]
fn email_context_returns_ev_parking_spot_for_arrival() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let response = email_context(
                ctx,
                EmailContextArgs {
                    template_key: Some(EmailTemplateKey::ArrivalDay),
                    locale: None,
                    ..Default::default()
                },
            )
            .expect("emailContext");
            assert_eq!(response.ev_parking_spot.as_deref(), Some("P2 / Place 14"));
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

/// A host writing in English: the French spot and instructions stay.
#[test]
#[serial]
fn a_save_in_english_keeps_the_french() {
    assert_eq!(
        config_save::localized_paths(EMISSIONS),
        ["booking_note", "instructions", "spot_label"]
    );
    let stored = json!({
        "spot_label": { "fr": "Place 14", "en": "Spot 14" },
        "instructions": { "fr": "À gauche après la barrière" },
        "charger_pin": "4821",
        "reveal_policy": "always"
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|mut ctx| {
            ctx.locale = "en-US".into();
            let surface = render_host_main(ctx).expect("host main");
            let sent = config_save::form_args(&surface);
            assert_eq!(sent["spot_label"], "Spot 14");
            assert_eq!(sent["instructions"], "À gauche après la barrière");

            let saved = config_save::save(EMISSIONS, &surface, &stored, "en");
            assert_eq!(saved["spot_label"], stored["spot_label"]);
            assert_eq!(saved["instructions"]["fr"], "À gauche après la barrière");
            assert_eq!(saved["charger_pin"], "4821");
        });
}

/// The arrival email reads the spot in the guest's language.
#[test]
#[serial]
fn email_context_picks_the_email_locale() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "spot_label": { "fr": "Place 14", "en": "Spot 14" } }))
        .run(|ctx| {
            let response = email_context(
                ctx,
                EmailContextArgs {
                    template_key: Some(EmailTemplateKey::Arrival),
                    locale: Some("en-GB".into()),
                    ..Default::default()
                },
            )
            .expect("emailContext");
            assert_eq!(response.ev_parking_spot.as_deref(), Some("Spot 14"));
        });
}

/// Un seul code : une seule tuile, et elle tient toute la largeur (§2.3).
///
/// La grille est en `auto-fit / minmax(min(100%, 130px), 1fr)` : à un enfant, la colonne unique
/// prend la ligne entière. Rien à calculer côté module — mais rien ne le disait non plus.
#[test]
#[serial]
fn one_code_draws_one_tile() {
    for (config, expected, absent) in [
        (
            json!({ "spot_label": "P2", "parking_code": "1234", "reveal_policy": "always" }),
            "i18n:guest.parkingCode",
            "i18n:guest.chargerPin",
        ),
        (
            json!({ "spot_label": "P2", "charger_pin": "4821", "reveal_policy": "always" }),
            "i18n:guest.chargerPin",
            "i18n:guest.parkingCode",
        ),
    ] {
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&config)
            .run(|ctx| {
                let json =
                    serde_json::to_string(&render_home_card(ctx).expect("carte")).expect("json");
                assert!(json.contains(expected), "{json}");
                assert!(!json.contains(absent), "{json}");
                assert_eq!(json.matches("\"layout\":\"tile\"").count(), 1, "{json}");
            });
    }
}

/// Une place sans code : la place suffit, et aucune grille vide ne s'ouvre sous elle (§2.3).
#[test]
#[serial]
fn a_spot_without_a_code_draws_no_grid() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "spot_label": "Place 14, niveau -1",
            "instructions": "Rampe à droite",
            "reveal_policy": "always"
        }))
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("carte");
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("Place 14, niveau -1"));
            assert!(!json.contains("\"Grid\""), "{json}");
            assert!(!json.contains("\"layout\":\"tile\""), "{json}");
        });
}

/// La barrière porte l'icône de la maquette — une voiture, pas un cadenas.
#[test]
#[serial]
fn the_barrier_tile_carries_the_car_glyph() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&always_reveal_config())
        .run(|ctx| {
            let json = serde_json::to_string(&render_home_card(ctx).expect("carte")).expect("json");
            assert!(json.contains("\"icon\":\"car\""), "{json}");
        });
}

/// La borne placée part sur la Carte du livret ; sans position, rien.
#[test]
#[serial]
fn a_placed_charger_reaches_the_booklet_map() {
    let markers = |config: Value| {
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&config)
            .run(|ctx| ev_parking::map_markers(ctx).expect("markers").markers)
    };
    assert_eq!(
        markers(json!({ "spot_label": "Place 8", "charger_lat": "43.58", "charger_lng": 7.12 }))
            .len(),
        1
    );
    assert!(markers(json!({ "spot_label": "Place 8" })).is_empty());
}

/// Spec §2 : trois `Section`, les trois cartes de la coque, dans l'ordre — jamais une `Card`.
#[test]
#[serial]
fn the_host_page_is_three_sections_in_the_spec_order() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let json = serde_json::to_string(&render_host_main(ctx).expect("host")).unwrap();
            let at = |key: &str| json.find(key).unwrap_or_else(|| panic!("{key}: {json}"));
            assert!(at("host.section.spot") < at("host.section.charger"));
            assert!(at("host.section.charger") < at("host.section.pricing"));
            assert_eq!(json.matches("\"type\":\"Section\"").count(), 3, "{json}");
            assert!(!json.contains("\"type\":\"Card\""), "{json}");
            // La prise en quatre cartes, la puissance en − / +, le câble en deux choix.
            assert_eq!(json.matches("\"type\":\"SelectableCard\"").count(), 4);
            assert!(json.contains("\"type\":\"NumberInput\""), "{json}");
            assert!(json.contains("i18n:host.cable.no"), "{json}");
            // Masqués tant que leur interrupteur ne les ouvre pas.
            assert!(json.contains("\"visibleWhen\":\"barrier_same_as_access=false\""));
            assert!(json.contains("\"visibleWhen\":\"booking_required=true\""));
        });
}

/// Le prix n'apparaît qu'avec une recharge payante — le choix du brouillon d'abord.
#[test]
#[serial]
fn the_price_shows_once_the_charge_is_not_included() {
    let has_price = |config: Value, input: Value| {
        MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&config)
            .run(|mut ctx| {
                ctx.input = input;
                let json = serde_json::to_string(&render_host_main(ctx).expect("host")).unwrap();
                json.contains("\"name\":\"price\"")
            })
    };
    assert!(!has_price(sample_config(), json!({})));
    assert!(has_price(sample_config(), json!({ "pricing": "per_stay" })));
    let mut paid = sample_config();
    paid["pricing"] = json!("on_site");
    assert!(has_price(paid.clone(), json!({})));
    assert!(!has_price(paid, json!({ "pricing": "included" })));
}

/// Un code de 3 à 12 caractères, message exact sous le champ ; celui d'Accès ne se valide pas ici.
#[test]
#[serial]
fn a_code_out_of_bounds_is_refused_under_its_field() {
    let errors = |config: Value| {
        MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&config)
            .run(|ctx| {
                let json = serde_json::to_value(render_host_main(ctx).expect("host")).unwrap();
                let mut found = Vec::new();
                collect_errors(&json, &mut found);
                found
            })
    };
    let message = "Le code doit contenir de 3 à 12 caractères.".to_string();
    assert_eq!(
        errors(json!({ "spot_label": "P2", "parking_code": "12", "charger_pin": "1234567890123" })),
        [
            ("parking_code".to_string(), message.clone()),
            ("charger_pin".to_string(), message)
        ]
    );
    assert!(errors(json!({
        "spot_label": "P2", "parking_code": "12", "barrier_same_as_access": true
    }))
    .is_empty());
    assert!(errors(
        json!({ "spot_label": "P2", "parking_code": "123", "charger_pin": "AB12CD34EF56" })
    )
    .is_empty());
}

fn collect_errors(node: &Value, out: &mut Vec<(String, String)>) {
    match node {
        Value::Object(map) => {
            if let (Some(Value::String(name)), Some(Value::String(error))) =
                (map.get("name"), map.get("error"))
            {
                out.push((name.clone(), error.clone()));
            }
            map.values().for_each(|v| collect_errors(v, out));
        }
        Value::Array(items) => items.iter().for_each(|v| collect_errors(v, out)),
        _ => {}
    }
}

/// Même code que le parking d'Accès : aucune tuile barrière, aucun code, un renvoi vers Accès.
#[test]
#[serial]
fn the_access_barrier_code_is_pointed_to_never_shown() {
    let mut config = always_reveal_config();
    config["barrier_same_as_access"] = json!(true);
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .run(|ctx| {
            let json = serde_json::to_string(&render_explore_detail(ctx).expect("detail")).unwrap();
            assert!(json.contains("i18n:guest.barrier.inAccess"), "{json}");
            assert!(!json.contains("i18n:guest.parkingCode"), "{json}");
            assert!(!json.contains("1234"), "{json}");
            assert!(json.contains("4821"), "{json}");
        });
}

/// Câble non fourni : le bandeau nomme la prise quand elle se nomme (§9 cas 4).
#[test]
#[serial]
fn no_cable_names_the_plug_to_bring() {
    let banner = |charger_type: &str| {
        let mut config = always_reveal_config();
        config["cable_provided"] = json!(false);
        config["charger_type"] = json!(charger_type);
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_translation("host.chargerType.label.type2", "Type 2")
            .with_translation("guest.cable.bringPlug", "Apportez votre câble {plug}.")
            .with_config(&config)
            .run(|ctx| serde_json::to_string(&render_explore_detail(ctx).expect("detail")).unwrap())
    };
    assert!(banner("type2").contains("Apportez votre câble Type 2."));
    assert!(banner("domestic").contains("i18n:guest.cable.bring\""));
}

/// La puissance en − / +, et l'estimation de recharge en aide dès qu'elle est renseignée (§4).
#[test]
#[serial]
fn the_power_hint_estimates_the_charge() {
    let mut config = sample_config();
    config["power_kw"] = json!("7");
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .with_translation("guest.power.estimate", "Environ {hours} h pour 40 kWh.")
        .run(|ctx| {
            let json = serde_json::to_string(&render_host_main(ctx).expect("host")).unwrap();
            assert!(json.contains("Environ 6 h pour 40 kWh."), "{json}");
            assert!(json.contains("\"value\":7.0"), "{json}");
        });
}

/// Sans épingle, la carte s'ouvre sur le logement (§4).
#[test]
#[serial]
fn the_pin_starts_on_the_property() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .with_coordinates(Some(portaki_sdk::sdui::GeoPoint::new(43.55, 7.12)))
        .run(|ctx| {
            let json = serde_json::to_string(&render_host_main(ctx).expect("host")).unwrap();
            assert!(json.contains("\"lat\":43.55"), "{json}");
            assert!(json.contains("\"lng\":7.12"), "{json}");
        });
}
