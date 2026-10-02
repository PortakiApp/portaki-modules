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

/// Le tri décide *quelles* quatre règles la carte montre : une règle importante posée en
/// cinquième position passe devant les neutres (§2.8).
#[test]
#[serial]
fn the_card_lifts_the_important_rule_above_the_neutral_ones() {
    reset_test_store();
    let five = json!({
        "items": [
            {"icon": "x", "title": "Pas de chaussures à l'intérieur"},
            {"icon": "users", "title": "Visiteurs en journée seulement"},
            {"icon": "minus", "title": "Barbecue à éteindre après usage"},
            {"icon": "check-circle", "title": "Vélos au garage"},
            {"icon": "clock-circle", "title": "Calme après 22 h", "status": "important"}
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
                    content_fr: five.clone(),
                    content_en: String::new(),
                },
            )
            .expect("save");
            let surface = render_home_card(ctx).expect("render");
            let json = serde_json::to_string(&surface).expect("json");

            assert!(json.contains("Calme après 22 h"), "{json}");
            // La dernière neutre cède sa place et attend dans la page.
            assert!(!json.contains("Vélos au garage"), "{json}");
        });
}

/// Les étiquettes : warning « Important », success « Autorisé », et rien du tout sur une neutre —
/// une étiquette « Normal » partout affaiblirait les deux autres (§2.8).
#[test]
#[serial]
fn only_important_and_allowed_rules_wear_a_badge() {
    reset_test_store();
    let three = json!({
        "items": [
            {"icon": "clock-circle", "title": "Calme après 22 h", "status": "important"},
            {"icon": "check-circle", "title": "Animaux bienvenus", "status": "allowed"},
            {"icon": "users", "title": "Six personnes maximum"}
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
                    content_fr: three.clone(),
                    content_en: String::new(),
                },
            )
            .expect("save");
            let tree = serde_json::to_value(render_home_card(ctx).expect("render")).expect("json");
            let rows = find_list_items(&tree);
            assert_eq!(rows.len(), 3, "{tree}");

            let badge = |row: &serde_json::Value| -> Option<(String, String)> {
                let b = row.get("trailing")?.get("badge")?;
                Some((
                    b.get("label")?.as_str()?.to_string(),
                    b.get("tone")?.as_str()?.to_string(),
                ))
            };
            // Le libellé passe par `t!`, donc il vaut la clé tant qu'aucun bundle n'est chargé —
            // c'est `previews.json` qui vérifie le texte rendu. Ici c'est le ton qui compte.
            assert_eq!(
                badge(&rows[0]),
                Some(("rule.status.important".into(), "warning".into())),
                "{tree}"
            );
            assert_eq!(
                badge(&rows[1]),
                Some(("rule.status.allowed".into(), "success".into())),
                "{tree}"
            );
            assert_eq!(badge(&rows[2]), None, "{tree}");
        });
}

/// Le détail groupe par thème, dans l'ordre où l'hôte a posé les règles ; un thème unique ne vaut
/// pas un en-tête (§2.8).
#[test]
#[serial]
fn the_detail_groups_by_theme_but_not_when_there_is_only_one() {
    reset_test_store();
    let grouped = json!({
        "items": [
            {"icon": "clock-circle", "title": "Calme après 22 h", "theme": "Voisinage"},
            {"icon": "x", "title": "Pas de verre au bord du bassin", "theme": "Piscine"},
            {"icon": "sun", "title": "Piscine de 8 h à 20 h", "theme": "piscine"}
        ]
    })
    .to_string();
    let single = json!({
        "items": [
            {"icon": "clock-circle", "title": "Calme après 22 h", "theme": "Voisinage"},
            {"icon": "users", "title": "Six personnes maximum", "theme": "Voisinage"}
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
                    content_fr: grouped.clone(),
                    content_en: String::new(),
                },
            )
            .expect("save");
            let json = serde_json::to_string(&render_explore_detail(ctx.clone()).expect("render"))
                .unwrap();
            // Deux thèmes, deux cartes titrées — et « piscine » rejoint « Piscine ».
            assert!(json.contains("\"title\":\"Voisinage\""), "{json}");
            assert_eq!(json.matches("\"title\":\"Piscine\"").count(), 1, "{json}");
            assert!(!json.contains("\"title\":\"piscine\""), "{json}");

            save_content(
                ctx.clone(),
                SaveContentArgs {
                    items: Vec::new(),
                    content_fr: single.clone(),
                    content_en: String::new(),
                },
            )
            .expect("save");
            let json = serde_json::to_string(&render_explore_detail(ctx).expect("render")).unwrap();
            assert!(
                !json.contains("Voisinage"),
                "un seul thème, pas d'en-tête : {json}"
            );
        });
}

/// Le statut ne dépend pas de la langue : l'hôte qui marque une règle importante en français la
/// retrouve importante en anglais, sinon le voyageur anglophone lit un règlement plus mou.
#[test]
#[serial]
fn the_status_follows_the_rule_into_every_language() {
    reset_test_store();
    let bilingual = json!({
        "items": [
            {"icon": "clock-circle", "title": "Calme après 22 h"},
            {"icon": "check-circle", "title": "Animaux bienvenus"}
        ]
    })
    .to_string();
    let en = json!({
        "items": [
            {"icon": "clock-circle", "title": "Quiet after 10 pm"},
            {"icon": "check-circle", "title": "Pets welcome"}
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
                    content_fr: bilingual.clone(),
                    content_en: en.clone(),
                },
            )
            .expect("save");
            // L'hôte repasse en français et pose les statuts.
            save_content(
                ctx.clone(),
                SaveContentArgs {
                    items: vec![
                        RuleItemInput {
                            icon: "clock-circle".into(),
                            title: "Calme après 22 h".into(),
                            status: "important".into(),
                            theme: "Voisinage".into(),
                            ..RuleItemInput::default()
                        },
                        RuleItemInput {
                            icon: "check-circle".into(),
                            title: "Animaux bienvenus".into(),
                            status: "allowed".into(),
                            theme: "Animaux".into(),
                            ..RuleItemInput::default()
                        },
                    ],
                    content_fr: String::new(),
                    content_en: String::new(),
                },
            )
            .expect("save");

            let view = get_content(
                ctx,
                GetContentArgs {
                    locale: Some("en-US".into()),
                },
            )
            .expect("view");
            assert_eq!(view.items[0].title, "Quiet after 10 pm");
            assert_eq!(view.items[0].status, rules::RuleStatus::Important);
            assert_eq!(view.items[1].status, rules::RuleStatus::Allowed);
            // Le thème, lui, est un texte : il reste dans la langue où il a été écrit.
            assert!(view.items[0].theme.is_empty(), "{:?}", view.items[0]);
        });
}

/// Toutes les `ListItem` d'un arbre rendu, dans l'ordre.
fn find_list_items(node: &serde_json::Value) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    if node.get("type").and_then(|t| t.as_str()) == Some("ListItem") {
        out.push(node.clone());
    }
    match node {
        serde_json::Value::Object(map) => {
            for value in map.values() {
                out.extend(find_list_items(value));
            }
        }
        serde_json::Value::Array(items) => {
            for value in items {
                out.extend(find_list_items(value));
            }
        }
        _ => {}
    }
    out
}

/// Le formulaire hôte porte bien les deux nouveaux champs par règle (§2.8).
#[test]
#[serial]
fn host_rows_carry_status_and_theme() {
    reset_test_store();
    MockContext::host()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let surface = render_host_main(ctx);
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("items.0.status"), "{json}");
            assert!(json.contains("items.0.theme"), "{json}");
            assert!(json.contains("rule.status.important"), "{json}");
            assert!(json.contains("host.rule.theme.hint"), "{json}");
        });
}
