//! `amenities.list` : le composteur, le tri sélectif dès une collecte ou un point d'apport, et
//! rien d'autre que l'id.

use portaki_sdk::capability;
use portaki_sdk::contracts::i18n::I18nText;
use portaki_test_utils::MockContext;
use serial_test::serial;
use waste_recycling::{amenities_list, BinRow, DropoffRow, ModuleConfig};

fn answer(config: &ModuleConfig) -> (Vec<String>, String) {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(config)
        .run(|ctx| {
            let answer = amenities_list(ctx).expect("amenities.list");
            let wire = serde_json::to_string(&answer).expect("json");
            (answer.amenities.into_iter().map(|a| a.id).collect(), wire)
        })
}

fn compost() -> ModuleConfig {
    ModuleConfig {
        has_collection: Some(false),
        compost_enabled: true,
        compost_location: I18nText::new("Fond du jardin, près du figuier", "By the fig tree"),
        compost_accepted: I18nText::new("Épluchures", "Peelings"),
        ..ModuleConfig::default()
    }
}

fn dropoff(glass: bool) -> DropoffRow {
    DropoffRow {
        id: "point-1".into(),
        title: I18nText::new("Conteneurs du parking Aubernon", "Aubernon car park"),
        address: Some("Rue Aubernon, Antibes".into()),
        lat: Some(43.58),
        lng: Some(7.12),
        accepts_glass: glass,
        ..DropoffRow::default()
    }
}

#[test]
#[serial]
fn compost_and_recycling() {
    let config = ModuleConfig {
        has_collection: Some(true),
        collects_tue: true,
        bin_room_code: "B7421".into(),
        bin_room_where: I18nText::new("Local au sous-sol", "Basement room"),
        bins: vec![BinRow {
            title: I18nText::new("Bac jaune", "Yellow bin"),
            items: I18nText::new("Cartons", "Cardboard"),
            ..BinRow::default()
        }],
        dropoff_points: vec![dropoff(true)],
        ..compost()
    };
    let (ids, wire) = answer(&config);
    assert_eq!(ids, ["compost", "recycling"]);
    for value in [
        "figuier",
        "fig tree",
        "Épluchures",
        "Aubernon",
        "43.58",
        "B7421",
        "sous-sol",
        "Bac jaune",
        "Cartons",
        "point-1",
    ] {
        assert!(!wire.contains(value), "{value} leaked: {wire}");
    }
}

#[test]
#[serial]
fn compost_only_once_located() {
    assert_eq!(answer(&compost()).0, ["compost"]);
    let unlocated = ModuleConfig {
        compost_location: I18nText::default(),
        ..compost()
    };
    assert!(answer(&unlocated).0.is_empty());
    let disabled = ModuleConfig {
        compost_enabled: false,
        ..compost()
    };
    assert!(answer(&disabled).0.is_empty());
}

#[test]
#[serial]
fn recycling_from_a_collection_day() {
    let config = ModuleConfig {
        collects_fri: true,
        ..ModuleConfig::default()
    };
    assert_eq!(answer(&config).0, ["recycling"]);
    let bin_day = ModuleConfig {
        bins: vec![BinRow {
            title: I18nText::new("Verre", "Glass"),
            days: vec!["mon".into()],
            ..BinRow::default()
        }],
        ..ModuleConfig::default()
    };
    assert_eq!(answer(&bin_day).0, ["recycling"]);
}

#[test]
#[serial]
fn recycling_from_a_dropoff_point() {
    let config = ModuleConfig {
        has_collection: Some(false),
        dropoff_points: vec![dropoff(true)],
        ..ModuleConfig::default()
    };
    assert_eq!(answer(&config).0, ["recycling"]);
}

#[test]
#[serial]
fn nothing_without_compost_collection_or_dropoff() {
    assert!(answer(&ModuleConfig::default()).0.is_empty());
    // Pas de ramassage : les jours cochés d'avant ne comptent plus.
    let no_collection = ModuleConfig {
        has_collection: Some(false),
        collects_mon: true,
        ..ModuleConfig::default()
    };
    assert!(answer(&no_collection).0.is_empty());
    // Un point qui n'accepte rien n'est pas un point d'apport.
    let empty_point = ModuleConfig {
        has_collection: Some(false),
        dropoff_points: vec![dropoff(false)],
        ..ModuleConfig::default()
    };
    assert!(answer(&empty_point).0.is_empty());
}
