//! `amenities.list` : la serrure connectée ou la boîte à clés selon la méthode, et rien d'autre
//! que l'id.

use access_guide::{amenities_list, HostConfig, PrimaryMethod};
use portaki_sdk::capability;
use portaki_sdk::contracts::i18n::I18nText;
use portaki_test_utils::MockContext;
use serial_test::serial;

fn answer(config: &HostConfig) -> (Vec<String>, String) {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(config)
        .run(|ctx| {
            let answer = amenities_list(ctx).expect("amenities.list");
            let wire = serde_json::to_string(&answer).expect("json");
            (answer.amenities.into_iter().map(|a| a.id).collect(), wire)
        })
}

fn method(method: PrimaryMethod) -> HostConfig {
    HostConfig {
        primary_method: method.as_wire().into(),
        keybox_location: I18nText::new("Derrière le volet bleu", "Behind the blue shutter"),
        keybox_code: "4821".into(),
        address: "12 chemin des Douaniers".into(),
        ..HostConfig::default()
    }
}

#[test]
#[serial]
fn a_keybox_is_a_key_box() {
    let (ids, wire) = answer(&method(PrimaryMethod::Keybox));
    assert_eq!(ids, ["key-box"]);
    for value in ["4821", "volet bleu", "blue shutter", "Douaniers"] {
        assert!(!wire.contains(value), "{value} leaked: {wire}");
    }
}

#[test]
#[serial]
fn a_smart_lock_is_a_smart_lock() {
    assert_eq!(answer(&method(PrimaryMethod::SmartLock)).0, ["smart-lock"]);
}

#[test]
#[serial]
fn nothing_for_the_other_methods() {
    assert!(answer(&HostConfig::default()).0.is_empty());
    for other in [
        PrimaryMethod::DoorCode,
        PrimaryMethod::InPerson,
        PrimaryMethod::BuildingStaff,
        PrimaryMethod::HostGreets,
        PrimaryMethod::Other,
    ] {
        assert!(answer(&method(other)).0.is_empty(), "{other:?}");
    }
}
