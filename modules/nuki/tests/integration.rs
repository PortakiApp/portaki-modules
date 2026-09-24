//! Integration-style unit tests with `portaki-test-utils`.

use nuki::{
    get_config, get_guest_credential, unlock, update_config, StayArgs, UpdateConfigArgs, SMART_LOCK,
};
use portaki_sdk::capability;
use portaki_sdk::contracts::smart_lock;
use portaki_sdk::prelude::{DateTime, Utc};
use portaki_test_utils::{Booking, MockContext, MockContextBuilder};
use serial_test::serial;

fn sample_config_bytes() -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "smartlock_id": "lock-abc",
        "keypad_code": "482910",
        "device_name": "Front door"
    }))
    .expect("config json")
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
        .with_kv("config", sample_config_bytes())
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
        .with_kv(
            "config",
            serde_json::to_vec(&serde_json::json!({
                "smartlock_id": "lock-abc",
                "keypad_code": "",
                "device_name": ""
            }))
            .unwrap(),
        )
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
        .with_kv("config", sample_config_bytes())
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
fn update_config_roundtrip() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            update_config(
                ctx.clone(),
                UpdateConfigArgs {
                    smartlock_id: "nuki-99".into(),
                    keypad_code: "123456".into(),
                    device_name: "Entry".into(),
                },
            )
            .expect("updateConfig");
            let config = get_config(ctx).expect("getConfig");
            assert_eq!(config.smartlock_id, "nuki-99");
            assert_eq!(config.keypad_code, "123456");
            assert_eq!(config.device_name, "Entry");
        });
}
