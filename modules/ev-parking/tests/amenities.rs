//! `amenities.list` : la borne dès qu'une place est renseignée, la prise en précision neutre, et
//! rien d'autre.

use ev_parking::{amenities_list, ModuleConfig};
use portaki_sdk::capability;
use portaki_sdk::contracts::amenities::ProvidedAmenity;
use portaki_sdk::contracts::i18n::I18nText;
use portaki_test_utils::MockContext;
use serial_test::serial;

fn answer(config: &ModuleConfig) -> (Vec<ProvidedAmenity>, String) {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(config)
        .run(|ctx| {
            let answer = amenities_list(ctx).expect("amenities.list");
            let wire = serde_json::to_string(&answer).expect("json");
            (answer.amenities, wire)
        })
}

fn spot(charger_type: &str) -> ModuleConfig {
    ModuleConfig {
        spot_label: I18nText::new("Place 14, niveau -2", "Spot 14, level -2"),
        charger_pin: "7731".into(),
        parking_code: "A2B4".into(),
        map_url: Some("https://maps.example.com/borne".into()),
        instructions: Some(I18nText::new("Badge sur le pilier", "Badge on the pillar")),
        charger_type: charger_type.into(),
        power_kw: 7.4,
        pricing: "per_kwh".into(),
        price: "0,25 €/kWh".into(),
        charger_lat: Some(43.58),
        charger_lng: Some(7.12),
        charger_address: "Parking Aubernon".into(),
        ..ModuleConfig::default()
    }
}

#[test]
#[serial]
fn a_charger_once_a_spot_is_set() {
    let (amenities, wire) = answer(&spot("type2"));
    assert_eq!(amenities.len(), 1);
    assert_eq!(amenities[0].id, "ev-charger");
    assert_eq!(amenities[0].detail.as_deref(), Some("Type 2"));
    for value in [
        "Place 14",
        "Spot 14",
        "7731",
        "A2B4",
        "maps.example.com",
        "pilier",
        "7.4",
        "0,25",
        "43.58",
        "Aubernon",
    ] {
        assert!(!wire.contains(value), "{value} leaked: {wire}");
    }
}

#[test]
#[serial]
fn the_plug_only_when_chosen_and_neutral() {
    assert_eq!(answer(&spot("ccs")).0[0].detail.as_deref(), Some("CCS"));
    for no_detail in ["", "domestic", "other"] {
        let (amenities, _) = answer(&spot(no_detail));
        assert_eq!(amenities[0].id, "ev-charger");
        assert_eq!(amenities[0].detail, None, "{no_detail}");
    }
}

#[test]
#[serial]
fn nothing_without_a_spot() {
    assert!(answer(&ModuleConfig::default()).0.is_empty());
    let codes_only = ModuleConfig {
        spot_label: I18nText::default(),
        ..spot("type2")
    };
    assert!(answer(&codes_only).0.is_empty());
}
