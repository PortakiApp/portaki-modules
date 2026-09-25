//! Integration-style unit tests with `portaki-test-utils`.

use nuki::{
    get_guest_credential, publish_readiness, render_host_main, unlock, ModuleConfig, StayArgs,
    SMART_LOCK,
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
    use portaki_sdk::context::CapabilityGrant;
    use portaki_sdk::host::with_host;

    let (mut ctx, host) = guest_at("2026-06-02T12:00:00Z")
        .with_connector_response("nuki", "remote_unlock", "{}")
        .build();
    ctx.capabilities.push(CapabilityGrant {
        id: "external.nuki.byok".into(),
    });

    with_host(host, ctx.clone(), || {
        let result = unlock(ctx, StayArgs { stay_id: None }).expect("unlock");
        assert!(result.ok);
        assert_eq!(result.mode, "remote");
        assert!(result.code.is_empty());
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
    use portaki_sdk::context::CapabilityGrant;
    use portaki_sdk::host::with_host;

    let ready = |config: ModuleConfig, byok: bool| {
        let (mut ctx, host) = MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&config)
            .build();
        if byok {
            ctx.capabilities.push(CapabilityGrant {
                id: "external.nuki.byok".into(),
            });
        }
        with_host(host, ctx.clone(), || {
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
