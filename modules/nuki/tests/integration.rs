//! Integration-style unit tests with `portaki-test-utils`.

use nuki::{
    get_guest_credential, publish_readiness, render_host_main, render_host_stay, unlock,
    ModuleConfig, StayArgs, SMART_LOCK,
};
use portaki_sdk::capability;
use portaki_sdk::contracts::smart_lock;
use portaki_sdk::prelude::{DateTime, Utc};
use portaki_test_utils::{Booking, MockContext, MockContextBuilder};
use serial_test::serial;

#[path = "../../../support/config_form.rs"]
mod config_form;

fn sample_config() -> ModuleConfig {
    ModuleConfig {
        smartlock_id: "lock-abc".into(),
        keypad_code: "482910".into(),
        device_name: "Front door".into(),
        code_per_stay: true,
    }
}

fn at(instant: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(instant)
        .expect("date")
        .with_timezone(&Utc)
}

/// A guest of the default booking (15:00Z 1 June → 10:00Z 8 June), at `now`.
fn guest_at(now: &str) -> MockContextBuilder {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .with_stay(Booking::default())
        .with_now(at(now))
}

#[test]
fn module_declares_smart_lock_capability() {
    assert_eq!(SMART_LOCK, smart_lock::CAPABILITY.as_str());
    assert_eq!(smart_lock::UNLOCK.as_str(), "unlock");
    assert_eq!(
        smart_lock::GET_GUEST_CREDENTIAL.as_str(),
        "getGuestCredential"
    );
}

#[test]
#[serial]
fn get_guest_credential_returns_keypad_code() {
    guest_at("2026-06-02T12:00:00Z").run(|ctx| {
        let cred =
            get_guest_credential(ctx, StayArgs { stay_id: None }).expect("getGuestCredential");
        assert_eq!(cred.credential_type, "keypad");
        assert_eq!(cred.code, "482910");
        assert_eq!(cred.smartlock_id, "lock-abc");
    });
}

#[test]
#[serial]
fn get_guest_credential_errors_without_keypad() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&ModuleConfig {
            smartlock_id: "lock-abc".into(),
            ..ModuleConfig::default()
        })
        .with_stay(Booking::default())
        .with_now(at("2026-06-02T12:00:00Z"))
        .run(|ctx| {
            let err =
                get_guest_credential(ctx, StayArgs { stay_id: None }).expect_err("expected error");
            assert!(err.to_string().contains("keypad_code"));
        });
}

#[test]
#[serial]
fn unlock_returns_credential_fallback() {
    guest_at("2026-06-02T12:00:00Z").run(|ctx| {
        let result = unlock(
            ctx,
            StayArgs {
                stay_id: Some("stay-1".into()),
            },
        )
        .expect("unlock");
        assert!(result.ok);
        assert_eq!(result.mode, "credential_fallback");
        assert_eq!(result.code, "482910");
    });
}

#[test]
#[serial]
fn unlock_prefers_remote_when_byok_granted() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE, capability::external::NUKI_BYOK])
        .with_config(&sample_config())
        .with_stay(Booking::default())
        .with_now(at("2026-06-02T12:00:00Z"))
        .with_connector_response("nuki", "remote_unlock", "{}")
        .run(|ctx| {
            let result = unlock(ctx, StayArgs { stay_id: None }).expect("unlock");
            assert!(result.ok);
            assert_eq!(result.mode, "remote");
            assert!(result.code.is_empty());
        });
}

/// No Nuki key: the keypad code still answers. A door with neither key nor code stays an error
/// — that is a real dead end, not a foreseen absence.
#[test]
#[serial]
fn without_the_nuki_key_the_keypad_answers_and_a_dead_door_errors() {
    guest_at("2026-06-02T12:00:00Z").run(|ctx| {
        let result = unlock(ctx, StayArgs::default()).expect("unlock");
        assert!(result.ok);
        assert_eq!(result.mode, "credential_fallback");
        assert_eq!(result.code, "482910");
    });

    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&ModuleConfig {
            smartlock_id: "lock-abc".into(),
            ..ModuleConfig::default()
        })
        .with_stay(Booking::default())
        .with_now(at("2026-06-02T12:00:00Z"))
        .run(|ctx| {
            let err = unlock(ctx, StayArgs::default()).expect_err("no way in");
            assert!(err.to_string().contains("unlock unavailable"), "{err}");
        });
}

/* Le voyageur est devant la porte : ce qu'il lit doit lui dire quoi faire. Une serrure qui n'a
pas répondu le dit et donne le code ; une serrure qu'on n'a jamais appelée donne le code ; une
porte qui s'ouvre le dit. Sans cette phrase, le bouton semblait ne rien faire. */
#[test]
#[serial]
fn an_offline_lock_says_so_and_hands_the_keypad_code() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE, capability::external::NUKI_BYOK])
        .with_config(&sample_config())
        .with_stay(Booking::default())
        .with_now(at("2026-06-02T12:00:00Z"))
        .with_connector_error("nuki", "remote_unlock", "lock_offline")
        .with_translation(
            "guest.unlock.offline",
            "La serrure n’a pas répondu. Tapez le code {code} sur le clavier.",
        )
        .run(|ctx| {
            let result = unlock(ctx, StayArgs::default()).expect("unlock");
            assert_eq!(result.mode, "credential_fallback");
            assert_eq!(
                result.guest_notice,
                "La serrure n’a pas répondu. Tapez le code 482910 sur le clavier."
            );
        });
}

#[test]
#[serial]
fn each_outcome_names_its_own_sentence() {
    // Jamais appelée : le code, sans parler d'un échec qui n'a pas eu lieu.
    guest_at("2026-06-02T12:00:00Z").run(|ctx| {
        let result = unlock(ctx, StayArgs::default()).expect("unlock");
        assert_eq!(result.guest_notice, "guest.unlock.keypad");
    });
    // Ouverte à distance : rien à taper.
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE, capability::external::NUKI_BYOK])
        .with_config(&sample_config())
        .with_stay(Booking::default())
        .with_now(at("2026-06-02T12:00:00Z"))
        .with_connector_response("nuki", "remote_unlock", "{}")
        .run(|ctx| {
            let result = unlock(ctx, StayArgs::default()).expect("unlock");
            assert_eq!(result.guest_notice, "guest.unlock.opened");
        });
    // Et le code du clavier se lit aussi quand le voyageur le demande.
    guest_at("2026-06-02T12:00:00Z").run(|ctx| {
        let cred = get_guest_credential(ctx, StayArgs::default()).expect("credential");
        assert_eq!(cred.guest_notice, "guest.credential.keypad");
    });
}

#[test]
#[serial]
fn the_lock_answers_from_8h_before_checkin_until_checkout() {
    for now in ["2026-06-01T07:00:00Z", "2026-06-08T10:00:00Z"] {
        guest_at(now).run(|ctx| {
            unlock(ctx.clone(), StayArgs::default()).expect("unlock");
            get_guest_credential(ctx, StayArgs::default()).expect("getGuestCredential");
        });
    }
}

#[test]
#[serial]
fn the_lock_refuses_outside_the_stay_window() {
    for now in ["2026-06-01T06:59:59Z", "2026-06-08T10:00:01Z"] {
        guest_at(now).run(|ctx| {
            let err = unlock(ctx.clone(), StayArgs::default()).expect_err("unlock");
            assert!(err.to_string().contains("outside_stay_window"), "{err}");
            let err = get_guest_credential(ctx, StayArgs::default()).expect_err("credential");
            assert!(err.to_string().contains("outside_stay_window"), "{err}");
        });
    }
}

#[test]
#[serial]
fn the_lock_refuses_without_a_stay_or_its_dates() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let err = unlock(ctx, StayArgs::default()).expect_err("no stay");
            assert!(err.to_string().contains("outside_stay_window"), "{err}");
        });
    let mut undated: portaki_sdk::context::StayContext = Booking::default().into();
    undated.checkout_at = None;
    guest_at("2026-06-02T12:00:00Z")
        .with_stay(undated)
        .run(|ctx| {
            let err = unlock(ctx, StayArgs::default()).expect_err("no checkout");
            assert!(err.to_string().contains("outside_stay_window"), "{err}");
        });
}

#[test]
#[serial]
fn the_host_form_sends_the_declared_keys_but_never_the_code() {
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
            // The keypad code is a secret: blank in the form keeps it.
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("lock-abc"));
            assert!(!json.contains("482910"));
        });
}

/// A keypad code, or remote unlock (Nuki Web key granted + lock ID): nothing else will do.
#[test]
#[serial]
fn publication_needs_a_keypad_code_or_remote_unlock() {
    let ready = |config: ModuleConfig, byok: bool| {
        let capabilities: &[capability::CapabilityId] = if byok {
            &[capability::core::STORAGE, capability::external::NUKI_BYOK]
        } else {
            &[capability::core::STORAGE]
        };
        MockContext::host()
            .with_capabilities(capabilities)
            .with_config(&config)
            .run(|ctx| {
                let items = publish_readiness(ctx).expect("publishReadiness").items;
                assert_eq!(items.len(), 1);
                items[0].ok
            })
    };
    let lock_only = ModuleConfig {
        smartlock_id: "lock-abc".into(),
        keypad_code: " ".into(),
        ..ModuleConfig::default()
    };
    assert!(ready(sample_config(), false));
    assert!(ready(lock_only.clone(), true));
    assert!(!ready(lock_only, false));
    assert!(!ready(ModuleConfig::default(), true));
}

// --- Un code par séjour (spec Nuki §2.2, §9 #1) ---------------------------------------------

/// A guest with the Nuki Web key granted, during the default booking.
fn byok_guest() -> MockContextBuilder {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE, capability::external::NUKI_BYOK])
        .with_config(&sample_config())
        .with_stay(Booking::default())
        .with_now(at("2026-06-02T12:00:00Z"))
}

fn calls_of(
    host: &portaki_test_utils::MockHostFunctions,
    operation: &str,
) -> Vec<serde_json::Value> {
    host.connector_calls()
        .into_iter()
        .filter(|call| call.operation == operation)
        .map(|call| serde_json::from_str(&call.args_json).expect("args json"))
        .collect()
}

#[test]
#[serial]
fn the_first_reveal_creates_the_stays_code_from_arrival_to_departure() {
    byok_guest()
        .with_connector_response("nuki", "list_auths", "[]")
        .with_connector_response("nuki", "create_auth", "{}")
        .run_with(|ctx, host| {
            let stay_id = ctx.stay.as_ref().expect("stay").stay_id;
            let cred = get_guest_credential(ctx, StayArgs::default()).expect("credential");
            assert_ne!(cred.code, "482910", "not the shared code");
            assert_eq!(cred.code.len(), 6);
            assert!(!cred.code.contains('0'));

            let created = calls_of(host, "create_auth");
            assert_eq!(created.len(), 1);
            let auth = &created[0];
            assert_eq!(auth["smartlockId"], "lock-abc");
            assert_eq!(
                auth["name"],
                format!("Portaki {}", &stay_id.simple().to_string()[..8])
            );
            assert_eq!(auth["type"], 13);
            assert_eq!(auth["code"].to_string(), cred.code);
            assert_eq!(auth["allowedFromDate"], "2026-06-01T15:00:00.000Z");
            assert_eq!(auth["allowedUntilDate"], "2026-06-08T10:00:00.000Z");
        });
}

#[test]
#[serial]
fn a_code_already_on_the_lock_is_given_again_never_recreated() {
    let booking = Booking::default();
    let name = format!("Portaki {}", &booking.id.simple().to_string()[..8]);
    let auths = format!(
        r#"[{{"name":"Host","type":13,"code":777777}},{{"name":"{name}","type":13,"code":482913}}]"#
    );
    byok_guest()
        .with_stay(booking)
        .with_connector_response("nuki", "list_auths", auths)
        .run_with(|ctx, host| {
            let cred = get_guest_credential(ctx, StayArgs::default()).expect("credential");
            assert_eq!(cred.code, "482913");
            assert!(calls_of(host, "create_auth").is_empty());
        });
}

/// §9 #1 : Nuki ne répond pas, le voyageur a le code de secours.
#[test]
#[serial]
fn nuki_failing_falls_back_to_the_shared_code() {
    byok_guest()
        .with_connector_error("nuki", "list_auths", "connector_egress_failed")
        .run(|ctx| {
            let cred = get_guest_credential(ctx, StayArgs::default()).expect("credential");
            assert_eq!(cred.code, "482910");
        });
    byok_guest()
        .with_connector_response("nuki", "list_auths", "[]")
        .with_connector_error("nuki", "create_auth", "connector_http_error")
        .run(|ctx| {
            let cred = get_guest_credential(ctx, StayArgs::default()).expect("credential");
            assert_eq!(cred.code, "482910");
        });
}

#[test]
#[serial]
fn without_code_per_stay_the_lock_is_never_asked() {
    byok_guest()
        .with_config(&ModuleConfig {
            code_per_stay: false,
            ..sample_config()
        })
        .run_with(|ctx, host| {
            let cred = get_guest_credential(ctx, StayArgs::default()).expect("credential");
            assert_eq!(cred.code, "482910");
            assert!(host.connector_calls().is_empty());
        });
}

/// La serrure n'a pas répondu à « Ouvrir » : un code créé maintenant ne l'atteindrait pas.
#[test]
#[serial]
fn an_unanswered_unlock_hands_the_shared_code_without_creating_one() {
    byok_guest()
        .with_connector_error("nuki", "remote_unlock", "lock_offline")
        .with_connector_response("nuki", "list_auths", "[]")
        .run_with(|ctx, host| {
            let result = unlock(ctx, StayArgs::default()).expect("unlock");
            assert_eq!(result.code, "482910");
            assert!(calls_of(host, "list_auths").is_empty());
        });
}

fn host_stay(capabilities: &[capability::CapabilityId]) -> MockContextBuilder {
    MockContext::host()
        .with_capabilities(capabilities)
        .with_config(&sample_config())
}

/// Renders the stay encart with `input` (the builder has no input setter). Property time: Paris.
fn render_stay(builder: MockContextBuilder, input: serde_json::Value) -> String {
    builder.run(|mut ctx| {
        ctx.input = input;
        serde_json::to_string(&render_host_stay(ctx).expect("stay")).unwrap()
    })
}

fn stay_input(stay_id: &str) -> serde_json::Value {
    serde_json::json!({
        "stayId": stay_id,
        "stay": { "checkIn": "2026-08-24T14:00:00Z", "checkOut": "2026-08-29T08:00:00Z" }
    })
}

#[test]
#[serial]
fn the_stay_encart_shows_the_window_and_the_code() {
    let json = render_stay(
        host_stay(&[capability::core::STORAGE, capability::external::NUKI_BYOK])
            .with_translation("host.stay.window", "Accès Nuki actif du {from} au {until}")
            .with_translation("host.stay.code", "Code {code}")
            .with_connector_response(
                "nuki",
                "list_auths",
                r#"[{"name":"Portaki 1a2b3c4d","type":13,"code":482913}]"#,
            ),
        stay_input("1a2b3c4d-0000-4000-8000-000000000000"),
    );
    assert!(
        json.contains("Accès Nuki actif du 24/08 16:00 au 29/08 10:00"),
        "{json}"
    );
    assert!(json.contains("Code 482913"), "{json}");
}

#[test]
#[serial]
fn the_stay_encart_says_why_there_is_no_code() {
    let byok = &[capability::core::STORAGE, capability::external::NUKI_BYOK];
    let input = || stay_input("1a2b3c4d-0000-4000-8000-000000000000");
    let pending = render_stay(
        host_stay(byok).with_connector_response("nuki", "list_auths", "[]"),
        input(),
    );
    assert!(pending.contains("host.stay.pending"), "{pending}");
    let offline = render_stay(
        host_stay(byok).with_connector_error("nuki", "list_auths", "connector_egress_failed"),
        input(),
    );
    assert!(offline.contains("host.stay.offline"), "{offline}");
    let shared = render_stay(host_stay(&[capability::core::STORAGE]), input());
    assert!(shared.contains("host.stay.shared"), "{shared}");
    let unknown = render_stay(
        host_stay(byok),
        serde_json::json!({ "stayId": "1a2b3c4d-0000-4000-8000-000000000000" }),
    );
    assert!(unknown.contains("host.stay.unknown"), "{unknown}");
}
