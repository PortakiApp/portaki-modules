//! Integration-style unit tests with `portaki-test-utils`.

use portaki_test_utils::{MockContext, Property, SurfaceAssertions};
use serde_json::json;

use train::{render_explore_detail, render_home_card, render_upcoming_card};

#[test]
fn home_card_shows_board_glance() {
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let card = render_home_card(ctx).expect("render");
            assert!(SurfaceAssertions::new(&card).contains_type("Card"));
            // Des lignes, pas des `TimedEntry` : un départ s'ouvre et porte son état.
            assert!(SurfaceAssertions::new(&card).contains_type("ListItem"));
            assert!(!SurfaceAssertions::new(&card).contains_type("TimedEntry"));

            let card_json = serde_json::to_string(&card).expect("json");
            assert!(card_json.contains("\"type\":\"openOverlay\""));
            assert!(card_json.contains("explore.detail"));
            assert!(card_json.contains("Nice-Ville"));
        });
}

#[test]
fn upcoming_card_is_compact_with_single_headline() {
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let card = render_upcoming_card(ctx).expect("render");
            assert!(SurfaceAssertions::new(&card).contains_type("Card"));
            assert!(SurfaceAssertions::new(&card).contains_type("Text"));
            // Compact: no full departure board on the prep card.
            assert!(!SurfaceAssertions::new(&card).contains_type("ListItem"));

            let card_json = serde_json::to_string(&card).expect("json");
            assert!(card_json.contains("upcoming.card"));
            assert!(card_json.contains("Nice-Ville"));
        });
}

#[test]
fn explore_detail_defaults_to_nice_ville_and_offers_the_sense_and_the_station() {
    MockContext::guest()
        .with_property(Property::default())
        .run(|ctx| {
            let detail = render_explore_detail(ctx).expect("render");
            // Le sens en segmenté, la gare en champ de recherche : les deux questions qu'on se
            // pose sur un quai, et que des pastilles de filtre ne posaient pas.
            assert!(SurfaceAssertions::new(&detail).contains_type("ChoiceList"));
            assert!(SurfaceAssertions::new(&detail).contains_type("ListItem"));
            assert!(!SurfaceAssertions::new(&detail).contains_type("FilterChip"));
            assert!(!SurfaceAssertions::new(&detail).contains_type("TimedEntry"));

            let detail_json = serde_json::to_string(&detail).expect("json");
            assert!(detail_json.contains("\"segmented\""));
            assert!(detail_json.contains("\"combobox\""));
            assert!(detail_json.contains("Nice-Ville"));
            assert!(detail_json.contains("Cannes"));
            assert!(detail_json.contains("Monaco"));
            assert!(detail_json.contains("Grasse"));
            // La fiche d'un départ est au bout de chaque ligne.
            assert!(detail_json.contains("train/nice-ville-"));
        });
}

#[test]
fn explore_detail_honors_dest_param() {
    MockContext::guest()
        .with_property(Property::default())
        .run(|mut ctx| {
            ctx.input = json!({ "dest": "Cannes" });
            let detail = render_explore_detail(ctx).expect("render");
            let detail_json = serde_json::to_string(&detail).expect("json");
            assert!(detail_json.contains("\"value\":\"Cannes\""));
            assert!(detail_json.contains("quai 3"));
        });
}

#[test]
fn explore_detail_falls_back_to_default_on_unknown_dest() {
    MockContext::guest()
        .with_property(Property::default())
        .run(|mut ctx| {
            ctx.input = json!({ "dest": "Marseille" });
            let detail = render_explore_detail(ctx).expect("render");
            let detail_json = serde_json::to_string(&detail).expect("json");
            assert!(detail_json.contains("\"value\":\"Nice-Ville\""));
        });
}
