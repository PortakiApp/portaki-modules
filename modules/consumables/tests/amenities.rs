//! `amenities.list` : les produits de base dès qu'un produit nommé est au catalogue, et aucun nom
//! de produit.

use consumables::{
    amenities_list, replace_items, reset_test_store, ConsumableItemInput, ReplaceItemsArgs,
};
use portaki_test_utils::MockContext;
use serial_test::serial;

fn item(fr: &str, en: &str) -> ConsumableItemInput {
    ConsumableItemInput {
        emoji: "☕".into(),
        label: String::new(),
        label_fr: fr.into(),
        label_en: en.into(),
        sort_order: 0,
        low_threshold: 0,
    }
}

fn answer(items: Vec<ConsumableItemInput>) -> (Vec<String>, String) {
    reset_test_store();
    MockContext::host().run(|ctx| {
        if !items.is_empty() {
            replace_items(
                ctx.clone(),
                ReplaceItemsArgs {
                    items,
                    items_json: None,
                },
            )
            .expect("replace");
        }
        let answer = amenities_list(ctx).expect("amenities.list");
        let wire = serde_json::to_string(&answer).expect("json");
        (answer.amenities.into_iter().map(|a| a.id).collect(), wire)
    })
}

#[test]
#[serial]
fn pantry_basics_with_a_named_product() {
    let (ids, wire) = answer(vec![
        item("Café moulu", "Ground coffee"),
        item("Huile d'olive", ""),
    ]);
    assert_eq!(ids, ["pantry-basics"]);
    for value in ["Café", "coffee", "olive", "☕"] {
        assert!(!wire.contains(value), "{value} leaked: {wire}");
    }
}

#[test]
#[serial]
fn nothing_with_an_empty_catalog() {
    assert!(answer(Vec::new()).0.is_empty());
    assert!(answer(vec![item("", "")]).0.is_empty());
}
