//! `amenities.list` : le wifi dès qu'un réseau est configuré, et rien d'autre que l'id.

use portaki_sdk::capability;
use portaki_test_utils::MockContext;
use serial_test::serial;
use wifi_guest::{amenities_list, ModuleConfig, Network};

fn ids(config: &ModuleConfig) -> (Vec<String>, String) {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(config)
        .run(|ctx| {
            let answer = amenities_list(ctx).expect("amenities.list");
            let wire = serde_json::to_string(&answer).expect("json");
            (answer.amenities.into_iter().map(|a| a.id).collect(), wire)
        })
}

#[test]
#[serial]
fn wifi_with_a_network() {
    let config = ModuleConfig {
        networks: vec![Network {
            id: "main".into(),
            ssid: "Islette_Guest".into(),
            password: "soleil2026".into(),
            ..Network::default()
        }],
        ..ModuleConfig::default()
    };
    let (ids, wire) = ids(&config);
    assert_eq!(ids, ["wifi"]);
    for value in ["Islette_Guest", "soleil2026", "main"] {
        assert!(!wire.contains(value), "{value} leaked: {wire}");
    }
}

#[test]
#[serial]
fn wifi_from_a_flat_config_of_before() {
    let config = ModuleConfig {
        legacy_ssid: "Ancien".into(),
        ..ModuleConfig::default()
    };
    assert_eq!(ids(&config).0, ["wifi"]);
}

#[test]
#[serial]
fn nothing_without_a_network() {
    let blank = ModuleConfig {
        networks: vec![Network::default()],
        ..ModuleConfig::default()
    };
    assert!(ids(&ModuleConfig::default()).0.is_empty());
    assert!(ids(&blank).0.is_empty());
}
