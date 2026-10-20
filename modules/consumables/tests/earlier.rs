//! La reprise du catalogue d'avant : des lignes `consumable_item` et les réglages en KV tels que
//! l'ancien module les a écrits (capturés depuis son code sur `main`, `tests/earlier/*.json`),
//! relus une fois.

use consumables::{Channel, Products, Restocks, SubmitArgs};
use portaki_test_utils::{MockContext, MockContextBuilder};
use serde_json::{json, Value};
use serial_test::serial;

fn stored(name: &str) -> Value {
    let path = format!("{}/tests/earlier/{name}.json", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// Les lignes (à l'envers, l'ordre vient de `sort_order`) et les valeurs KV capturées.
fn seeded(builder: MockContextBuilder, name: &str) -> MockContextBuilder {
    let data = stored(name);
    let mut rows = data["items"].as_array().unwrap().clone();
    rows.reverse();
    let mut builder = builder.with_rows("ConsumableItem", rows);
    for key in ["restock_delay", "settings"] {
        if !data["kv"][key].is_null() {
            builder = builder.with_kv(key, serde_json::to_vec(&data["kv"][key]).unwrap());
        }
    }
    builder
}

fn old_ids(name: &str) -> Vec<String> {
    stored(name)["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_str().unwrap().to_string())
        .collect()
}

#[test]
#[serial]
fn the_catalog_and_its_settings_become_the_collection() {
    seeded(MockContext::host(), "catalog").run(|ctx| {
        let products = Products::load(&ctx).unwrap();
        let rows = products.rows();
        let seen: Vec<(&str, &str, &str, &str, &str)> = rows
            .iter()
            .map(|p| {
                (
                    p.label.get("fr"),
                    p.label.get("en"),
                    p.emoji.as_str(),
                    p.location.get("fr"),
                    p.location.get("en"),
                )
            })
            .collect();
        assert_eq!(
            seen,
            [
                (
                    "Papier toilette",
                    "Toilet paper",
                    "🧻",
                    "Sous l'évier",
                    "Sous l'évier"
                ),
                ("Café", "Coffee", "☕", "Placard de gauche", "Left cupboard"),
                ("Sel", "Sel", "", "", ""),
            ]
        );
        let ids: Vec<String> = rows.iter().map(|p| p.id.clone()).collect();
        assert_eq!(ids, old_ids("catalog"), "a request keeps its product");
        let meta = products.meta();
        assert_eq!(meta.requests_enabled, Some(false));
        assert_eq!(meta.max_requests, Some(8));
        assert_eq!(meta.notify, Some(vec![Channel::Email, Channel::App]));
        assert_eq!(meta.restock_delay.get("fr"), "sous 24 h");
        assert_eq!(meta.restock_delay.get("en"), "within 24 hours");
    });
}

#[test]
#[serial]
fn a_plain_french_label_keeps_its_english_and_requests_stay_linked() {
    seeded(MockContext::guest(), "plain").run(|ctx| {
        let products = Products::load(&ctx).unwrap();
        let labels: Vec<(&str, &str)> = products
            .rows()
            .iter()
            .map(|p| (p.label.get("fr"), p.label.get("en")))
            .collect();
        assert_eq!(
            labels,
            [("Thé", "Tea"), ("Sucre", "Sucre"), ("Huile", "Oil")]
        );
        // Rien en KV : les réglages par défaut.
        assert_eq!(products.meta().requests_enabled, None);
        assert_eq!(products.meta().notify, None);
        let tea = products.rows()[0].uuid();
        assert_eq!(tea.to_string(), old_ids("plain")[0]);
        consumables::__portaki_requests_submit(
            ctx.clone(),
            serde_json::to_value(SubmitArgs {
                item_ids: vec![tea],
                item_id: None,
                level: "missing".into(),
                note: None,
            })
            .unwrap(),
        )
        .unwrap();
        let rows = Restocks::list(&ctx).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].item.item_id, tea.to_string());
    });
}

#[test]
#[serial]
fn taken_over_once_and_never_over_the_collection() {
    seeded(MockContext::host(), "catalog")
        .with_resource(
            "products",
            json!([{ "id": "x", "label": { "fr": "Déjà là" } }]),
            json!({}),
        )
        .run(|ctx| assert_eq!(Products::load(&ctx).unwrap().len(), 1));
    seeded(MockContext::host(), "catalog").run(|ctx| {
        let mut products = Products::load(&ctx).unwrap();
        products.rows_mut().clear();
        Products::save(&products).unwrap();
        assert!(
            Products::load(&ctx).unwrap().is_empty(),
            "an emptied catalog stays empty"
        );
    });
}
