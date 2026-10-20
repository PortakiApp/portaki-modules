//! La reprise du règlement d'avant : des lignes `rules_content` telles que l'ancien module les a
//! écrites (capturées depuis son code sur `main`, `tests/earlier/*.json`), relues une fois.

use portaki_test_utils::MockContext;
use rules::{render_home_card, Rules, Theme};
use serde_json::{json, Value};

fn stored(name: &str) -> Value {
    let path = format!("{}/tests/earlier/{name}.json", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn the_multilingual_bundle_becomes_the_collection() {
    MockContext::guest()
        .with_rows("RulesContent", vec![stored("bundle")])
        .run(|ctx| {
            let rules = Rules::load(&ctx).unwrap();
            let titles: Vec<(&str, &str)> = rules
                .rows()
                .iter()
                .map(|r| (r.title.get("fr"), r.title.get("en")))
                .collect();
            assert_eq!(
                titles,
                [
                    ("Calme après 22 h", "Quiet after 10 pm"),
                    ("Pas de fête", "No parties"),
                    ("Animaux bienvenus", "Animaux bienvenus"),
                ]
            );
            assert_eq!(rules.rows()[0].id, "r0");
            assert_eq!(rules.rows()[0].detail.get("fr"), "Pensez aux voisins.");
            assert_eq!(rules.rows()[1].theme, Theme::Parties);
            assert_eq!(rules.meta().card_limit, 3);
            let card = serde_json::to_string(&render_home_card(ctx).unwrap()).unwrap();
            assert!(card.contains("Calme après 22 h"), "{card}");
        });
}

#[test]
fn the_older_format_gets_ids_and_a_free_theme_becomes_other() {
    MockContext::guest()
        .with_rows("RulesContent", vec![stored("dual")])
        .run(|ctx| {
            let rules = Rules::load(&ctx).unwrap();
            assert_eq!(rules.len(), 1);
            let rule = &rules.rows()[0];
            assert_eq!(
                (rule.id.as_str(), rule.title.get("en")),
                ("r0", "Quiet at night")
            );
            assert_eq!(rule.theme, Theme::Other);
        });
}

#[test]
fn taken_over_once_and_never_over_the_collection() {
    MockContext::host()
        .with_rows("RulesContent", vec![stored("bundle")])
        .with_resource(
            "rules",
            json!([{ "id": "x", "title": { "fr": "Déjà là" } }]),
            json!({}),
        )
        .run(|ctx| assert_eq!(Rules::load(&ctx).unwrap().len(), 1));
    MockContext::host()
        .with_rows("RulesContent", vec![stored("bundle")])
        .run(|ctx| {
            let mut rules = Rules::load(&ctx).unwrap();
            rules.rows_mut().clear();
            Rules::save(&rules).unwrap();
            assert!(
                Rules::load(&ctx).unwrap().is_empty(),
                "an emptied rulebook stays empty"
            );
        });
}
