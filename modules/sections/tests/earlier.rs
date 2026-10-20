//! La reprise des sections d'avant : des lignes `SectionItem` / `SectionItemLocale` telles que
//! l'ancien module les a écrites (capturées depuis son code sur `main`, `tests/earlier/*.json`),
//! relues une fois.

use portaki_test_utils::MockContext;
use sections::{render_home_card, Sections};
use serde_json::{json, Value};

fn stored(name: &str) -> Vec<Value> {
    let path = format!("{}/tests/earlier/{name}.json", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn seeded(ctx: MockContext) -> MockContext {
    ctx.with_rows("SectionItem", stored("items"))
        .with_rows("SectionItemLocale", stored("locales"))
}

#[test]
fn the_old_sections_become_the_collection_in_booklet_order() {
    seeded(MockContext::guest().with_locale("en-US")).run(|ctx| {
        let sections = Sections::load(&ctx).unwrap();
        let ids: Vec<&str> = sections.rows().iter().map(|s| s.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "1425cb84-50d2-409d-9470-43cea81e7717",
                "438a1307-8206-43a9-b424-a7190ce456ca",
                "be7a73db-fd26-435e-ada4-4730e24d2d76",
                "776d7599-490a-4e6b-8430-bf20f48fab2d",
            ],
            "the emptied section is not taken over"
        );
        let rows = sections.rows();
        assert!(rows[0].title.is_blank());
        assert_eq!(
            rows[0].body.get("fr"),
            "Le code du portail est dans la boîte à clés."
        );
        assert_eq!(
            (rows[1].title.get("fr"), rows[1].title.get("en")),
            ("Bienvenue", "Welcome")
        );
        assert_eq!(
            rows[1].body.get("en"),
            "Welcome to L'Islette!\n\nMake yourself at home."
        );
        assert!(rows[2].body.get("fr").contains("terrasse vue mer"));
        assert!(rows[2].body.get("en").contains("sea-view terrace"));
        assert_eq!(rows[3].title.get("de"), "Willkommen");
        let card = serde_json::to_string(&render_home_card(ctx).unwrap()).unwrap();
        assert!(card.contains("Make yourself at home."), "{card}");
    });
}

#[test]
fn taken_over_once_and_never_over_the_collection() {
    seeded(MockContext::host())
        .with_resource(
            "sections",
            json!([{ "id": "x", "title": { "fr": "Déjà là" } }]),
            json!({}),
        )
        .run(|ctx| assert_eq!(Sections::load(&ctx).unwrap().len(), 1));
    seeded(MockContext::host()).run(|ctx| {
        let mut sections = Sections::load(&ctx).unwrap();
        assert_eq!(sections.len(), 4);
        sections.rows_mut().clear();
        Sections::save(&sections).unwrap();
        assert!(
            Sections::load(&ctx).unwrap().is_empty(),
            "an emptied booklet stays empty"
        );
    });
}
