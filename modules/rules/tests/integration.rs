//! Integration-style unit tests with `portaki-test-utils`.

#![allow(clippy::disallowed_methods)] // tests natifs : l'horloge du système y est disponible

use chrono::Utc;
use portaki_sdk::capability;
use serial_test::serial;
use uuid::Uuid;

use portaki_test_utils::{MockContext, Property, SurfaceAssertions};
use serde_json::json;

use rules::{
    get_content, publish_readiness, render_explore_detail, render_home_card, render_host_main,
    reset_test_store, save_content, update_config, GetContentArgs, RuleItemInput, RulesContent,
    SaveContentArgs,
};

fn sample_payload() -> String {
    json!({
        "items": [
            {"icon": "clock-circle", "title": "Calme après 22 h", "subtitle": "Merci pour le voisinage"},
            {"icon": "x", "title": "Logement non-fumeur", "subtitle": "Terrasse autorisée"},
            {"icon": "users", "title": "Pas de fête ni d'événement", "subtitle": ""},
            {"icon": "check-circle", "title": "Animaux bienvenus", "subtitle": "Prévenez-nous"}
        ]
    })
    .to_string()
}

#[test]
#[serial]
fn home_card_renders_empty_without_content() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("render");
            assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
        });
}

#[test]
#[serial]
fn home_card_renders_list_items_with_content() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            save_content(
                ctx.clone(),
                SaveContentArgs {
                    items: Vec::new(),
                    content_fr: sample_payload(),
                    content_en: sample_payload(),
                },
            )
            .expect("save");
            let surface = render_home_card(ctx).expect("render");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
        });
}

/// Six règles : la carte en montre quatre et propose le reste (§2.8).
#[test]
#[serial]
fn the_card_offers_the_rules_it_does_not_show() {
    reset_test_store();
    let six = json!({
        "items": [
            {"icon": "clock-circle", "title": "Calme après 22 h"},
            {"icon": "x", "title": "Logement non-fumeur"},
            {"icon": "users", "title": "Pas de fête"},
            {"icon": "check-circle", "title": "Animaux bienvenus"},
            {"icon": "key", "title": "Rendre les clés avant 11 h"},
            {"icon": "trash", "title": "Sortir les poubelles le mardi"}
        ]
    })
    .to_string();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            save_content(
                ctx.clone(),
                SaveContentArgs {
                    items: Vec::new(),
                    content_fr: six.clone(),
                    content_en: six.clone(),
                },
            )
            .expect("save");
            let surface = render_home_card(ctx).expect("render");
            let json = serde_json::to_string(&surface).expect("json");

            assert!(json.contains("Calme après 22 h"));
            // La cinquième et la sixième attendent dans la page.
            assert!(!json.contains("Rendre les clés"), "{json}");
            assert!(
                SurfaceAssertions::new(&surface).contains_type("Button"),
                "{json}"
            );
        });
}

/// Quatre règles ou moins : pas de bouton. Il ouvrirait une page identique à la carte (§2.8).
#[test]
#[serial]
fn no_button_when_the_card_already_shows_every_rule() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            save_content(
                ctx.clone(),
                SaveContentArgs {
                    items: Vec::new(),
                    content_fr: sample_payload(),
                    content_en: sample_payload(),
                },
            )
            .expect("save");
            let surface = render_home_card(ctx).expect("render");

            assert!(!SurfaceAssertions::new(&surface).contains_type("Button"));
        });
}

#[test]
#[serial]
fn explore_detail_renders_full_list() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            save_content(
                ctx.clone(),
                SaveContentArgs {
                    items: Vec::new(),
                    content_fr: sample_payload(),
                    content_en: sample_payload(),
                },
            )
            .expect("save");
            let surface = render_explore_detail(ctx).expect("render");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Stack"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("Calme après 22 h"));
        });
}

#[test]
#[serial]
fn get_content_returns_saved_items() {
    reset_test_store();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            save_content(
                ctx.clone(),
                SaveContentArgs {
                    items: Vec::new(),
                    content_fr: sample_payload(),
                    content_en: String::new(),
                },
            )
            .expect("save");
            let view = get_content(
                ctx,
                GetContentArgs {
                    locale: Some("fr-FR".into()),
                },
            )
            .expect("get");
            assert_eq!(view.items.len(), 4);
            assert_eq!(view.items[0].title, "Calme après 22 h");
        });
}

#[test]
#[serial]
fn host_main_renders_rules_section_steplist() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            let surface = render_host_main(ctx);
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("StepList"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Form"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("host.section.title") || json.contains("Règles du logement"));
            assert!(json.contains("host.rules.add") || json.contains("Ajouter une règle"));
        });
}

#[test]
#[serial]
fn update_config_persists_items_for_locale() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .run(|ctx| {
            update_config(
                ctx.clone(),
                SaveContentArgs {
                    items: vec![RuleItemInput {
                        icon: "clock-circle".into(),
                        title: "Calme après 22 h".into(),
                        subtitle: "Merci pour le voisinage".into(),
                        ..Default::default()
                    }],
                    content_fr: String::new(),
                    content_en: String::new(),
                },
            )
            .expect("updateConfig");
            let view = get_content(
                ctx,
                GetContentArgs {
                    locale: Some("fr-FR".into()),
                },
            )
            .expect("get");
            assert_eq!(view.items.len(), 1);
            assert_eq!(view.items[0].title, "Calme après 22 h");
        });
}

#[test]
#[serial]
fn seed_row_shape_matches_entity() {
    let now = Utc::now();
    let row = RulesContent {
        id: Uuid::new_v4(),
        content_fr: sample_payload(),
        content_en: String::new(),
        created_at: now,
        updated_at: now,
    };
    assert!(!row.content_fr.is_empty());
}

#[test]
#[serial]
fn publish_readiness_requires_one_rule() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            assert!(
                !publish_readiness(ctx.clone())
                    .expect("publishReadiness")
                    .items[0]
                    .ok
            );
            save_content(
                ctx.clone(),
                SaveContentArgs {
                    items: Vec::new(),
                    content_fr: sample_payload(),
                    content_en: String::new(),
                },
            )
            .expect("save");
            assert!(publish_readiness(ctx).expect("publishReadiness").items[0].ok);
        });
}
