//! `amenities.list` : les produits de base dès qu'un produit nommé est au catalogue, et aucun nom
//! de produit.

mod sample;

use consumables::amenities_list;
use portaki_test_utils::MockContext;

fn answer(items: &[(&str, &str, &str)]) -> (Vec<String>, String) {
    sample::with(MockContext::host(), items).run(|ctx| {
        let answer = amenities_list(ctx).expect("amenities.list");
        let wire = serde_json::to_string(&answer).expect("json");
        (answer.amenities.into_iter().map(|a| a.id).collect(), wire)
    })
}

#[test]
fn pantry_basics_with_a_named_product() {
    let (ids, wire) = answer(&[
        ("Café moulu", "Ground coffee", "☕"),
        ("Huile d'olive", "", "☕"),
    ]);
    assert_eq!(ids, ["pantry-basics"]);
    for value in ["Café", "coffee", "olive", "☕"] {
        assert!(!wire.contains(value), "{value} leaked: {wire}");
    }
}

#[test]
fn nothing_with_an_empty_catalog() {
    assert!(answer(&[]).0.is_empty());
    assert!(answer(&[("", "", "☕")]).0.is_empty());
}
