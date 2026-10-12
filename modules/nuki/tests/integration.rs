//! Integration-style unit tests with `portaki-test-utils`.

use nuki::{
    get_guest_credential, publish_readiness, regenerate_code, render_host_main,
    render_host_regenerate, render_host_stay, timeline_tasks, unlock, ModuleConfig, RegenerateArgs,
    StayArgs, SMART_LOCK,
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

// --- Les codes expirés quittent la serrure ---------------------------------------------------

/// A keypad authorization as Nuki lists it.
fn auth(id: &str, name: &str, until: Option<&str>) -> serde_json::Value {
    let mut auth = serde_json::json!({ "id": id, "name": name, "type": 13, "code": 456789 });
    if let Some(until) = until {
        auth["allowedUntilDate"] = until.into();
    }
    auth
}

fn deleted_ids(host: &portaki_test_utils::MockHostFunctions) -> Vec<String> {
    calls_of(host, "delete_auth")
        .into_iter()
        .map(|args| {
            assert_eq!(args["smartlockId"], "lock-abc");
            args["authId"].as_str().expect("authId").to_string()
        })
        .collect()
}

/// Now is 2 June 12:00Z (`byok_guest`).
fn lock_with_codes() -> String {
    serde_json::json!([
        auth("expired", "Portaki 0a1b2c3d", Some("2026-05-30T10:00:00.000Z")),
        auth("active", "Portaki 4e5f6a7b", Some("2026-06-05T10:00:00.000Z")),
        auth("no-end", "Portaki 8c9d0e1f", None),
        auth("host", "Ménage", Some("2026-01-01T10:00:00.000Z")),
        auth("host-portaki", "Portaki ménage", Some("2026-01-01T10:00:00.000Z")),
        { "id": "fob", "name": "Portaki 22223333", "type": 2, "allowedUntilDate": "2026-01-01T10:00:00.000Z" },
    ])
    .to_string()
}

#[test]
#[serial]
fn expired_portaki_codes_are_deleted_before_the_stays_code_is_created() {
    byok_guest()
        .with_connector_response("nuki", "list_auths", lock_with_codes())
        .with_connector_response("nuki", "delete_auth", "{}")
        .with_connector_response("nuki", "create_auth", "{}")
        .run_with(|ctx, host| {
            get_guest_credential(ctx, StayArgs::default()).expect("credential");
            assert_eq!(deleted_ids(host), vec!["expired"]);
            let order: Vec<String> = host
                .connector_calls()
                .into_iter()
                .map(|call| call.operation)
                .collect();
            assert_eq!(order, ["list_auths", "delete_auth", "create_auth"]);
        });
}

/// An active code, one without an end date, the host's own (even named « Portaki … »), a fob:
/// never deleted.
#[test]
#[serial]
fn active_and_foreign_authorizations_are_never_deleted() {
    let booking = Booking::default();
    let mine = format!("Portaki {}", &booking.id.simple().to_string()[..8]);
    let auths = serde_json::json!([
        auth("mine", &mine, Some("2026-06-08T10:00:00.000Z")),
        auth("active", "Portaki 4e5f6a7b", Some("2026-06-05T10:00:00.000Z")),
        auth("no-end", "Portaki 8c9d0e1f", None),
        auth("unreadable", "Portaki 9a9a9a9a", Some("demain")),
        auth("host", "Ménage", Some("2026-01-01T10:00:00.000Z")),
        auth("host-portaki", "Portaki ménage", Some("2026-01-01T10:00:00.000Z")),
        { "id": "fob", "name": "Portaki 22223333", "type": 2, "allowedUntilDate": "2026-01-01T10:00:00.000Z" },
    ])
    .to_string();
    byok_guest()
        .with_stay(booking)
        .with_connector_response("nuki", "list_auths", auths)
        .with_connector_response("nuki", "delete_auth", "{}")
        .run_with(|ctx, host| {
            let cred = get_guest_credential(ctx, StayArgs::default()).expect("credential");
            assert_eq!(cred.code, "456789");
            assert!(deleted_ids(host).is_empty());
        });
}

/// Nuki refusing a delete costs the guest nothing: the stay's code is still created.
#[test]
#[serial]
fn a_refused_delete_still_gives_the_stays_code() {
    byok_guest()
        .with_connector_response("nuki", "list_auths", lock_with_codes())
        .with_connector_error("nuki", "delete_auth", "connector_http_error")
        .with_connector_response("nuki", "create_auth", "{}")
        .run_with(|ctx, host| {
            let cred = get_guest_credential(ctx, StayArgs::default()).expect("credential");
            assert_ne!(cred.code, "482910", "not the shared code");
            assert_eq!(deleted_ids(host), vec!["expired"]);
            assert_eq!(calls_of(host, "create_auth").len(), 1);
        });
}

/// The stay encart only reads the lock: it never deletes.
#[test]
#[serial]
fn the_stay_encart_never_deletes() {
    host_stay(&[capability::core::STORAGE, capability::external::NUKI_BYOK])
        .with_connector_response("nuki", "list_auths", lock_with_codes())
        .with_connector_response("nuki", "delete_auth", "{}")
        .run_with(|mut ctx, host| {
            ctx.input = stay_input(&Booking::default().id.to_string());
            render_host_stay(ctx).expect("render");
            assert_eq!(calls_of(host, "list_auths").len(), 1);
            assert!(deleted_ids(host).is_empty());
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

/// Le tiroir de la planche : « Serrure » dit si la clé Nuki Web est là, « Codes » avertit
/// toujours qu'un code par séjour écrit sur la serrure — même éteint.
#[test]
#[serial]
fn the_drawer_says_whether_the_nuki_web_key_is_there() {
    let render = |capabilities: &[capability::CapabilityId]| {
        MockContext::host()
            .with_capabilities(capabilities)
            .with_config(&ModuleConfig::default())
            .run(|ctx| serde_json::to_string(&render_host_main(ctx).expect("host main")).unwrap())
    };
    let with_key = render(&[capability::core::STORAGE, capability::external::NUKI_BYOK]);
    let without = render(&[capability::core::STORAGE]);
    assert!(with_key.contains("i18n:host.key.connected"), "{with_key}");
    assert!(!with_key.contains("i18n:host.key.missing"), "{with_key}");
    assert!(without.contains("i18n:host.key.missing"), "{without}");
    for json in [&with_key, &without] {
        assert!(json.contains("i18n:host.section.lock"), "{json}");
        assert!(json.contains("i18n:host.section.codes"), "{json}");
        assert!(json.contains("i18n:host.codePerStay.warning"), "{json}");
    }
}

/// Copiés vers un autre logement, la serrure et son nom sont marqués « À vérifier » ; le code
/// clavier et le choix d'un code par séjour suivent tels quels.
#[test]
fn the_lock_and_its_name_belong_to_the_property() {
    let dir = concat!(env!("OUT_DIR"), "/portaki-emissions");
    let path = std::fs::read_dir(dir)
        .expect("emissions")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("config-")
        })
        .expect("config emission");
    let config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let local: Vec<&str> = config["fields"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|field| field["local"] == true)
        .filter_map(|field| field["key"].as_str())
        .collect();
    assert_eq!(local, ["smartlock_id", "device_name"]);
}

// --- « Régénérer le code » (spec Nuki §1, `stay-action`) ---------------------------------------

/// Two codes under the stay's name — Nuki lists the older last: the newest is the guest's.
#[test]
#[serial]
fn after_a_regeneration_the_newest_code_is_the_guests() {
    let booking = Booking::default();
    let name = format!("Portaki {}", &booking.id.simple().to_string()[..8]);
    let auths = format!(
        r#"[{{"name":"{name}","type":13,"code":654321,"creationDate":"2026-06-02T09:00:00.000Z"}},
            {{"name":"{name}","type":13,"code":482913,"creationDate":"2026-05-20T09:00:00.000Z"}}]"#
    );
    byok_guest()
        .with_stay(booking)
        .with_connector_response("nuki", "list_auths", auths)
        .run(|ctx| {
            let cred = get_guest_credential(ctx, StayArgs::default()).expect("credential");
            assert_eq!(cred.code, "654321");
        });
}

fn regenerate_args() -> RegenerateArgs {
    RegenerateArgs {
        stay_id: "1a2b3c4d-0000-4000-8000-000000000000".parse().unwrap(),
        check_in: at("2026-08-24T14:00:00Z"),
        check_out: at("2026-08-29T08:00:00Z"),
    }
}

#[test]
#[serial]
fn regenerating_creates_a_new_code_for_the_stay_window_even_with_one_on_the_lock() {
    host_stay(&[capability::core::STORAGE, capability::external::NUKI_BYOK])
        .with_now(at("2026-08-25T10:00:00Z"))
        .with_connector_response(
            "nuki",
            "list_auths",
            r#"[{"name":"Portaki 1a2b3c4d","type":13,"code":482913}]"#,
        )
        .with_connector_response("nuki", "create_auth", "{}")
        .run_with(|ctx, host| {
            regenerate_code(ctx, regenerate_args()).expect("regenerated");
            let created = calls_of(host, "create_auth");
            assert_eq!(created.len(), 1);
            assert_eq!(created[0]["name"], "Portaki 1a2b3c4d");
            assert_ne!(created[0]["code"], 482913);
            assert_eq!(created[0]["allowedFromDate"], "2026-08-24T14:00:00.000Z");
            assert_eq!(created[0]["allowedUntilDate"], "2026-08-29T08:00:00.000Z");
        });
}

#[test]
#[serial]
fn regenerating_is_refused_to_guests_without_per_stay_codes_and_after_departure() {
    // `with_capabilities` rebuilds the context: the guest identity is set back here.
    byok_guest().run_with(|mut ctx, host| {
        ctx.guest = Some(portaki_test_utils::GuestIdentityFixture::default().into());
        assert!(regenerate_code(ctx, regenerate_args()).is_err());
        assert!(host.connector_calls().is_empty());
    });
    host_stay(&[capability::core::STORAGE])
        .with_now(at("2026-08-25T10:00:00Z"))
        .run_with(|ctx, host| {
            assert!(regenerate_code(ctx, regenerate_args()).is_err());
            assert!(host.connector_calls().is_empty());
        });
    host_stay(&[capability::core::STORAGE, capability::external::NUKI_BYOK])
        .with_now(at("2026-08-29T09:00:00Z"))
        .run_with(|ctx, host| {
            assert!(regenerate_code(ctx, regenerate_args()).is_err());
            assert!(host.connector_calls().is_empty());
        });
}

fn render_regenerate(builder: MockContextBuilder, input: serde_json::Value) -> serde_json::Value {
    builder.run(|mut ctx| {
        ctx.input = input;
        serde_json::to_value(render_host_regenerate(ctx).expect("regenerate")).unwrap()
    })
}

/// The guest on site: the modal warns, says until when the old code works, and its one button
/// sends the stay and its window.
#[test]
#[serial]
fn the_regenerate_modal_warns_when_the_guest_is_there_and_sends_the_window() {
    let byok = &[capability::core::STORAGE, capability::external::NUKI_BYOK];
    let tree = render_regenerate(
        host_stay(byok)
            .with_now(at("2026-08-25T10:00:00Z"))
            .with_translation(
                "host.regenerate.body",
                "L’ancien reste valable jusqu’au {until}.",
            ),
        stay_input("1a2b3c4d-0000-4000-8000-000000000000"),
    );
    let json = tree.to_string();
    assert!(
        json.contains("L’ancien reste valable jusqu’au 29/08 10:00."),
        "{json}"
    );
    assert!(json.contains("i18n:host.regenerate.present"), "{json}");
    assert!(json.contains(r#""name":"regenerateCode""#), "{json}");
    assert!(
        json.contains(r#""checkOut":"2026-08-29T08:00:00Z""#),
        "{json}"
    );

    let before = render_regenerate(
        host_stay(byok).with_now(at("2026-08-20T10:00:00Z")),
        stay_input("1a2b3c4d-0000-4000-8000-000000000000"),
    )
    .to_string();
    assert!(!before.contains("host.regenerate.present"), "{before}");
    assert!(before.contains("regenerateCode"), "{before}");
}

#[test]
#[serial]
fn the_regenerate_modal_says_why_it_cannot() {
    let input = || stay_input("1a2b3c4d-0000-4000-8000-000000000000");
    let shared = render_regenerate(host_stay(&[capability::core::STORAGE]), input()).to_string();
    assert!(shared.contains("host.regenerate.unavailable"), "{shared}");
    assert!(!shared.contains("regenerateCode"), "{shared}");
    let byok = &[capability::core::STORAGE, capability::external::NUKI_BYOK];
    let ended = render_regenerate(
        host_stay(byok).with_now(at("2026-08-30T10:00:00Z")),
        input(),
    )
    .to_string();
    assert!(ended.contains("host.regenerate.ended"), "{ended}");
    assert!(!ended.contains("regenerateCode"), "{ended}");
    let unknown = render_regenerate(host_stay(byok), serde_json::json!({})).to_string();
    assert!(unknown.contains("host.stay.unknown"), "{unknown}");
}

// --- « Serrure hors ligne », « Batterie faible » dans À venir (spec Nuki §9) -----------------

fn lock_watch() -> MockContextBuilder {
    host_stay(&[capability::core::STORAGE, capability::external::NUKI_BYOK])
        .with_now(at("2026-08-20T10:00:00Z"))
}

fn week() -> portaki_sdk::contracts::timeline::TimelineTasksArgs {
    serde_json::from_value(serde_json::json!({
        "propertyId": "5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10",
        "from": "2026-08-17T00:00:00Z",
        "to": "2026-08-24T00:00:00Z",
        "stays": []
    }))
    .expect("args")
}

fn task_ids(builder: MockContextBuilder) -> Vec<String> {
    builder.run(|ctx| {
        timeline_tasks(ctx, week())
            .expect("tasks")
            .tasks
            .into_iter()
            .map(|task| task.id)
            .collect()
    })
}

#[test]
#[serial]
fn an_offline_lock_is_a_task_that_says_the_fallback_code_answers() {
    lock_watch()
        .with_connector_response("nuki", "get_smartlock", r#"{"serverState":4,"name":"x"}"#)
        .run_with(|ctx, host| {
            let tasks = timeline_tasks(ctx, week()).expect("tasks").tasks;
            assert_eq!(tasks.len(), 1);
            assert_eq!(tasks[0].id, "lock-offline");
            assert_eq!(tasks[0].title.fr, "Serrure hors ligne");
            assert_eq!(tasks[0].at, at("2026-08-20T10:00:00Z"));
            assert!(
                tasks[0].context.fr.contains("code de secours"),
                "{:?}",
                tasks[0].context
            );
            assert!(tasks[0].items.is_empty());
            assert_eq!(
                calls_of(host, "get_smartlock")[0]["smartlockId"],
                "lock-abc"
            );
        });
}

#[test]
#[serial]
fn a_low_or_critical_battery_is_a_task_with_its_charge() {
    lock_watch()
        .with_connector_response(
            "nuki",
            "get_smartlock",
            r#"{"serverState":0,"state":{"batteryCritical":false,"batteryCharge":15}}"#,
        )
        .run(|ctx| {
            let tasks = timeline_tasks(ctx, week()).expect("tasks").tasks;
            assert_eq!(tasks.len(), 1);
            assert_eq!(tasks[0].id, "battery-low");
            assert_eq!(tasks[0].title.fr, "Batterie faible");
            assert_eq!(tasks[0].context.fr, "15 % restants");
        });
    let critical = lock_watch().with_connector_response(
        "nuki",
        "get_smartlock",
        r#"{"serverState":4,"state":{"batteryCritical":true}}"#,
    );
    assert_eq!(task_ids(critical), ["lock-offline", "battery-low"]);
}

#[test]
#[serial]
fn a_healthy_lock_or_an_unanswering_nuki_is_no_task() {
    let healthy = lock_watch().with_connector_response(
        "nuki",
        "get_smartlock",
        r#"{"serverState":0,"state":{"batteryCritical":false,"batteryCharge":80}}"#,
    );
    assert!(task_ids(healthy).is_empty());
    let failing =
        lock_watch().with_connector_error("nuki", "get_smartlock", "connector_egress_failed");
    assert!(task_ids(failing).is_empty());
}

/// Sans clé Nuki Web, sans serrure, ou hors de la fenêtre qui contient maintenant : Nuki n'est
/// jamais appelé.
#[test]
#[serial]
fn the_lock_is_only_read_with_the_key_a_lock_and_the_current_window() {
    let cases = [
        host_stay(&[capability::core::STORAGE]).with_now(at("2026-08-20T10:00:00Z")),
        lock_watch().with_config(&ModuleConfig {
            smartlock_id: " ".into(),
            ..sample_config()
        }),
        lock_watch().with_now(at("2026-08-30T10:00:00Z")),
    ];
    for builder in cases {
        builder
            .with_connector_response("nuki", "get_smartlock", r#"{"serverState":4}"#)
            .run_with(|ctx, host| {
                assert!(timeline_tasks(ctx, week()).expect("tasks").tasks.is_empty());
                assert!(calls_of(host, "get_smartlock").is_empty());
            });
    }
}

/// À venir se relit à chaque focus : la serrure n'est lue qu'une fois par cache.
#[test]
#[serial]
fn the_lock_state_is_cached_between_two_reads() {
    lock_watch()
        .with_connector_response("nuki", "get_smartlock", r#"{"serverState":4}"#)
        .run_with(|ctx, host| {
            let first = timeline_tasks(ctx.clone(), week()).expect("first").tasks;
            let second = timeline_tasks(ctx, week()).expect("second").tasks;
            assert_eq!(first.len(), 1);
            assert_eq!(second.len(), 1);
            assert_eq!(calls_of(host, "get_smartlock").len(), 1);
        });
}

/// L'orchestrateur n'appelle `timelineTasks` que pour un module qui déclare la nav
/// `WorkspaceTimelineTask` : sans elle, « Serrure hors ligne » n'atteint jamais À venir.
#[test]
fn timeline_tasks_are_declared_to_the_dashboard() {
    let path = concat!(
        env!("OUT_DIR"),
        "/portaki-emissions/nav-tasks_HostPlacement__WorkspaceTimelineTask.json"
    );
    let raw = std::fs::read_to_string(path).expect("the tasks nav is emitted");
    let nav: serde_json::Value = serde_json::from_str(&raw).expect("nav json");
    assert_eq!(nav["placement"], "HostPlacement::WorkspaceTimelineTask");
    assert_eq!(nav["path"], "tasks");
    assert_eq!(nav["label_key"], "catalog.host.tasks");
}
