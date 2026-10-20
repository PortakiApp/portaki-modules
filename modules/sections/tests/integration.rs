//! Ce que les sections font de plus que la collection du SDK : la carte, la feuille, l'éditeur.
//! La fusion, l'import et les contrôles sont ceux du SDK, testés là-bas.

mod sample;

use portaki_test_utils::{MockContext, SurfaceAssertions};
use sections::{render_explore_sheet, render_home_card, render_host_main};
use serde_json::json;

fn tree(surface: &portaki_sdk::sdui::surface::Surface) -> String {
    serde_json::to_string(surface).unwrap()
}

#[test]
fn no_section_is_the_empty_state() {
    let surface = MockContext::guest().run(|ctx| render_home_card(ctx).unwrap());
    assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
}

#[test]
fn a_blank_document_is_no_section() {
    let blank = json!([{ "id": "s1", "title": {}, "body": { "fr": r#"{"type":"doc","content":[{"type":"paragraph"}]}"# } }]);
    let surface = MockContext::guest()
        .with_resource("sections", blank, json!({}))
        .run(|ctx| render_home_card(ctx).unwrap());
    assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
}

#[test]
fn the_card_shows_the_first_two_titled_and_the_sheet_signs_the_whole() {
    let (card, sheet) = sample::seeded_two(MockContext::guest()).run(|ctx| {
        (
            render_home_card(ctx.clone()).unwrap(),
            render_explore_sheet(ctx).unwrap(),
        )
    });
    let card = tree(&card);
    assert!(card.contains("Bienvenue à **L'Islette** !") && card.contains("L'appartement"));
    assert!(
        card.contains("Divider") && card.contains("bottomSheet"),
        "{card}"
    );
    let sheet = tree(&sheet);
    assert!(
        sheet.contains("2 chambres") && sheet.contains("signature"),
        "{sheet}"
    );
}

#[test]
fn a_lone_section_shows_its_body_only_in_the_guests_language() {
    let card = sample::seeded(MockContext::guest().with_locale("en-US"))
        .run(|ctx| render_home_card(ctx).unwrap());
    let card = tree(&card);
    assert!(card.contains("We are glad to have you."), "{card}");
    assert!(
        !card.contains("Welcome to L'Islette"),
        "the card titles itself"
    );
}

#[test]
fn a_document_renders_as_rich_text() {
    let doc = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Bonjour"}]}]}"#;
    let surface = MockContext::guest()
        .with_resource(
            "sections",
            json!([{ "id": "s1", "body": { "fr": doc } }]),
            json!({}),
        )
        .run(|ctx| render_home_card(ctx).unwrap());
    assert!(SurfaceAssertions::new(&surface).contains_type("RichText"));
}

#[test]
fn the_editor_opens_each_body_in_the_rich_editor() {
    let surface = sample::seeded_two(MockContext::host()).run(|ctx| render_host_main(ctx).unwrap());
    assert!(SurfaceAssertions::new(&surface).contains_type("RichTextEditor"));
    let tree = tree(&surface);
    assert!(tree.contains(r#""name":"sections.1.body""#), "{tree}");
    assert!(
        tree.contains("host.body.hint") && tree.contains("host.sections.add"),
        "{tree}"
    );
}
