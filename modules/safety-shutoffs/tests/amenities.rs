//! `amenities.list` : extincteur et détecteur de fumée d'après le type des organes affichés, et
//! rien d'autre que l'id.

use portaki_sdk::capability;
use portaki_sdk::contracts::i18n::I18nText;
use portaki_test_utils::MockContext;
use safety_shutoffs::{amenities_list, ModuleConfig, ShutoffRow};
use serial_test::serial;

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

fn row(kind: &str, location: &str) -> ShutoffRow {
    ShutoffRow {
        id: format!("row-{kind}"),
        kind: kind.into(),
        title: I18nText::new("Sous l'évier rouge", "Under the red sink"),
        location: I18nText::new(location, ""),
        instruction: I18nText::new("Tirer la goupille", "Pull the pin"),
        ..ShutoffRow::default()
    }
}

fn config(rows: Vec<ShutoffRow>) -> ModuleConfig {
    ModuleConfig {
        shutoffs: rows,
        general_note: I18nText::new("Numéro du syndic", "Building manager"),
    }
}

#[test]
#[serial]
fn an_extinguisher_and_a_smoke_detector() {
    let (ids, wire) = answer(&config(vec![
        row("water", "Cave"),
        row("smoke_detector", "Couloir"),
        row("extinguisher", "Cuisine, placard haut"),
    ]));
    assert_eq!(ids, ["fire-extinguisher", "smoke-detector"]);
    for value in [
        "évier",
        "red sink",
        "Couloir",
        "placard haut",
        "goupille",
        "Pull the pin",
        "syndic",
        "row-",
    ] {
        assert!(!wire.contains(value), "{value} leaked: {wire}");
    }
}

#[test]
#[serial]
fn each_on_its_own() {
    assert_eq!(
        answer(&config(vec![row("extinguisher", "Cuisine")])).0,
        ["fire-extinguisher"]
    );
    assert_eq!(
        answer(&config(vec![row("smoke_detector", "Couloir")])).0,
        ["smoke-detector"]
    );
}

#[test]
#[serial]
fn nothing_for_other_kinds_or_rows_without_a_location() {
    assert!(answer(&ModuleConfig::default()).0.is_empty());
    assert!(answer(&config(vec![
        row("electricity", "Entrée"),
        row("gas", "Cuisine"),
        row("other", "Garage"),
        row("unknown", "Cave"),
        row("extinguisher", ""),
        row("smoke_detector", "  "),
    ]))
    .0
    .is_empty());
}
