//! Integration-style unit tests with `portaki-test-utils`.

use chrono::{DateTime, Utc};
use portaki_sdk::capability;
use portaki_sdk::context::{HostProfile, StayContext};
use serde_json::{json, Value};
use serial_test::serial;

use portaki_test_utils::{MockContext, MockContextBuilder, Property, SurfaceAssertions};

use sections::{
    list_sections, publish_readiness, render_explore_sheet, render_home_card, render_host_main,
    reset_test_store, seed_legacy_section, ListSectionsArgs,
};

fn guest() -> MockContextBuilder {
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
}

fn host() -> MockContextBuilder {
    MockContext::host()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
}

fn two_sections() -> Value {
    json!({
        "show_from": "booking",
        "sections": [
            { "id": "a", "title": { "fr": "Bienvenue", "en": "Welcome" },
              "body": { "fr": "Bienvenue à L'Islette !", "en": "Welcome to L'Islette!" } },
            { "id": "b", "title": "L'appartement", "body": "2 chambres, terrasse vue mer.",
              "photo": "portaki-file:terrasse" }
        ]
    })
}

fn json_of<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("json")
}

fn utc(raw: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(raw)
        .expect("rfc3339")
        .with_timezone(&Utc)
}

#[test]
#[serial]
fn home_card_empty_without_content() {
    reset_test_store();
    guest().run(|ctx| {
        let surface = render_home_card(ctx).expect("render");
        assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
    });
}

#[test]
#[serial]
fn home_card_and_sheet_with_sections() {
    reset_test_store();
    guest().with_config(&two_sections()).run(|ctx| {
        let card = render_home_card(ctx.clone()).expect("render");
        assert!(SurfaceAssertions::new(&card).contains_type("Card"));
        assert!(SurfaceAssertions::new(&card).contains_type("Markdown"));
        assert!(!json_of(&card).contains("portaki-file:terrasse"));
        let sheet = render_explore_sheet(ctx).expect("render");
        let json = json_of(&sheet);
        assert!(json.contains("L'appartement"), "{json}");
        // La photo ouvre le panneau, pas la carte d'accueil.
        assert!(json.contains("portaki-file:terrasse"), "{json}");
    });
}

/// §8 : les sections des tables d'avant restent lisibles tant que l'hôte n'a rien enregistré.
#[test]
#[serial]
fn legacy_sections_stay_readable_until_the_first_save() {
    reset_test_store();
    seed_legacy_section(
        0,
        &[
            ("fr", "Bienvenue", "Bonjour à tous."),
            ("en", "Welcome", "Hello all."),
        ],
    );
    let listed = guest().run(|ctx| {
        list_sections(
            ctx,
            ListSectionsArgs {
                locale: Some("en-US".into()),
            },
        )
        .expect("list")
    });
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].title, "Welcome");

    // The host form shows them, and does not block on them.
    let (form, readiness) = host().run(|ctx| {
        (
            json_of(&render_host_main(ctx.clone()).expect("host")),
            publish_readiness(ctx).expect("readiness"),
        )
    });
    assert!(form.contains("Bienvenue"), "{form}");
    assert!(readiness.items.is_empty());

    // Saved, even empty, the list wins.
    let card = guest()
        .with_config(&json!({ "show_from": "booking", "sections": [] }))
        .run(|ctx| render_home_card(ctx).expect("render"));
    assert!(SurfaceAssertions::new(&card).contains_type("EmptyState"));
}

/// §2.1 : « La veille de l'arrivée » (défaut) — la ligne est absente avant minuit la veille.
#[test]
#[serial]
fn the_word_waits_for_show_from() {
    reset_test_store();
    let mut config = two_sections();
    config["show_from"] = json!("day_before");
    let stay = StayContext {
        checkin_at: Some(utc("2026-07-20T14:00:00Z")),
        ..StayContext::default()
    };
    let render = |now: &str| {
        guest()
            .with_config(&config)
            .with_stay(stay.clone())
            .with_now(utc(now))
            .run(|ctx| render_home_card(ctx).expect("render"))
    };
    let early = render("2026-07-15T10:00:00Z");
    assert!(SurfaceAssertions::new(&early).contains_type("EmptyState"));
    let eve = render("2026-07-19T10:00:00Z");
    assert!(json_of(&eve).contains("Bienvenue à L'Islette"));
}

#[test]
#[serial]
fn the_host_card_has_the_spec_blocks() {
    reset_test_store();
    let json = host()
        .with_config(&two_sections())
        .run(|ctx| json_of(&render_host_main(ctx).expect("host")));
    for expected in [
        "i18n:host.section.welcome",
        "i18n:host.section.sections",
        r#""type":"RadioGroup""#,
        r#""value":"booking""#,
        r#""type":"StepList""#,
        r#""itemKeyPrefix":"sections""#,
        "sections.0.title",
        "sections.1.photo",
        "i18n:host.sections.first",
        "i18n:host.signature.edit",
        "/settings?tab=profil",
        "i18n:host.sections.add",
    ] {
        assert!(json.contains(expected), "{expected} missing: {json}");
    }
    assert!(!json.contains("i18n:host.sections.template"), "{json}");
}

/// La signature : le prénom et la photo (ou les initiales) quand la plateforme les sert.
#[test]
#[serial]
fn the_signature_reads_the_host_profile() {
    reset_test_store();
    let json = host().run(|mut ctx| {
        ctx.host = Some(HostProfile {
            name: "Claire".into(),
            initials: Some("CL".into()),
            ..HostProfile::default()
        });
        json_of(&render_host_main(ctx).expect("host"))
    });
    assert!(json.contains(r#""value":"Claire""#), "{json}");
    assert!(json.contains(r#""initials":"CL""#), "{json}");
}

#[test]
#[serial]
fn an_empty_list_offers_a_template_filled_with_the_host_and_the_property() {
    reset_test_store();
    let empty = json!({ "sections": [] });
    let json = host()
        .with_config(&empty)
        .run(|ctx| json_of(&render_host_main(ctx).expect("host")));
    assert!(json.contains("i18n:host.sections.template"), "{json}");
    assert!(json.contains(r#""sections_count":1"#), "{json}");
    assert!(json.contains(r#""template":true"#), "{json}");
    assert!(!json.contains("sections.0.title"), "{json}");

    let json = host()
        .with_config(&empty)
        .with_translation("host.title.default", "Bienvenue")
        .with_translation(
            "host.template.body",
            "Bonjour, je suis {host} : bienvenue à {property} !",
        )
        .run(|mut ctx| {
            ctx.input = json!({ "sections_count": 1, "template": true });
            ctx.host = Some(HostProfile {
                name: "Claire".into(),
                ..HostProfile::default()
            });
            let property = ctx.property.name.clone();
            (json_of(&render_host_main(ctx).expect("host")), property)
        });
    let (json, property) = json;
    assert!(json.contains(r#""value":"Bienvenue""#), "{json}");
    assert!(
        json.contains(&format!("je suis Claire : bienvenue à {property} !")),
        "{json}"
    );
}

#[test]
#[serial]
fn ten_sections_and_no_add_button() {
    reset_test_store();
    let rows: Vec<Value> = (0..10)
        .map(|i| json!({ "id": format!("s{i}"), "title": format!("S{i}"), "body": "Texte." }))
        .collect();
    let json = host()
        .with_config(&json!({ "sections": rows }))
        .run(|ctx| json_of(&render_host_main(ctx).expect("host")));
    assert!(!json.contains("i18n:host.sections.add"), "{json}");
    assert!(json.contains("i18n:config.sections.max"), "{json}");
}

#[test]
#[serial]
fn a_broken_rule_shows_under_its_field_and_blocks_publish() {
    reset_test_store();
    let config = json!({ "sections": [{ "id": "a", "title": "", "body": "" }] });
    let (json, readiness) = host().with_config(&config).run(|ctx| {
        (
            json_of(&render_host_main(ctx.clone()).expect("host")),
            publish_readiness(ctx).expect("readiness"),
        )
    });
    // `title` est prérempli « Bienvenue » côté formulaire, mais rien n'est enregistré.
    assert!(json.contains("Donnez un titre à la section."), "{json}");
    assert!(json.contains("Écrivez quelques mots."), "{json}");
    let hints: Vec<&str> = readiness.items.iter().map(|i| i.hint.get("fr")).collect();
    assert_eq!(
        hints,
        ["Donnez un titre à la section.", "Écrivez quelques mots."]
    );
}

/// §3 : une première phrase de plus de 160 caractères — l'aide, sans bloquer.
#[test]
#[serial]
fn a_long_teaser_is_hinted_not_blocked() {
    reset_test_store();
    let long = format!("{}. Fin.", "mot ".repeat(45));
    let config = json!({ "sections": [{ "id": "a", "title": "Bienvenue", "body": long }] });
    let (json, readiness) = host().with_config(&config).run(|ctx| {
        (
            json_of(&render_host_main(ctx.clone()).expect("host")),
            publish_readiness(ctx).expect("readiness"),
        )
    });
    assert!(json.contains("i18n:config.body.teaser"), "{json}");
    assert_eq!(readiness.items.len(), 1);
    assert!(!readiness.items[0].ok);
}

/// Every `sections.N.key` input of the rendered form, with its seeded value — what the
/// dashboard holds before the first keystroke.
fn form_values(node: &Value, out: &mut Vec<(String, String)>) {
    match node {
        Value::Object(map) => {
            if let (Some(Value::String(name)), Some(Value::String(value))) =
                (map.get("name"), map.get("value"))
            {
                if name.starts_with("sections.") && !out.iter().any(|(n, _)| n == name) {
                    out.push((name.clone(), value.clone()));
                }
            }
            map.values().for_each(|child| form_values(child, out));
        }
        Value::Array(items) => items.iter().for_each(|child| form_values(child, out)),
        _ => {}
    }
}

/// What `updateConfig` stores from a host form in French: each row, its localized fields in
/// French only (the platform keeps the other languages of a row it already holds — none here).
fn saved_in_french(values: &[(String, String)]) -> Value {
    let mut rows: Vec<serde_json::Map<String, Value>> = Vec::new();
    for (name, value) in values {
        let mut parts = name.split('.').skip(1);
        let index: usize = parts.next().unwrap().parse().unwrap();
        let key = parts.next().unwrap();
        rows.resize_with(rows.len().max(index + 1), Default::default);
        let value = if key == "title" || key == "body" {
            json!({ "fr": value })
        } else {
            json!(value)
        };
        rows[index].insert(key.into(), value);
    }
    json!({ "sections": rows })
}

/// §8, sans perte : des sections d'avant, une seule modifiée → la config enregistrée les garde
/// toutes (ids conservés), la modifiée à jour, les autres langues reprises des tables.
#[test]
#[serial]
fn the_first_save_keeps_every_legacy_section() {
    reset_test_store();
    let first = seed_legacy_section(
        0,
        &[
            ("fr", "Bienvenue", "Bonjour à tous."),
            ("en", "Welcome", "Hello all."),
        ],
    );
    let second = seed_legacy_section(1, &[("fr", "Le jardin", "Les transats sont dehors.")]);

    let form =
        host().run(|ctx| serde_json::to_value(render_host_main(ctx).expect("host")).expect("json"));
    let mut values = Vec::new();
    form_values(&form, &mut values);
    // The one edit of the host: the second title.
    for (name, value) in values.iter_mut() {
        if name == "sections.1.title" {
            *value = "Le jardin et la piscine".into();
        }
    }
    let saved = saved_in_french(&values);

    let (fr, en) = [("fr-FR", "fr"), ("en-US", "en")]
        .map(|(locale, _)| {
            guest().with_config(&saved).run(|ctx| {
                list_sections(
                    ctx,
                    ListSectionsArgs {
                        locale: Some(locale.into()),
                    },
                )
                .expect("list")
            })
        })
        .into();
    let ids: Vec<String> = fr.iter().map(|s| s.id.clone()).collect();
    assert_eq!(ids, [first.to_string(), second.to_string()]);
    assert_eq!(fr[0].title, "Bienvenue");
    assert!(fr[0].body_markdown.contains("Bonjour à tous."));
    assert_eq!(fr[1].title, "Le jardin et la piscine");
    // Written in French only, the English of the first section comes from the tables.
    assert_eq!(en[0].title, "Welcome");
    assert!(en[0].body_markdown.contains("Hello all."));
}

/// Retirer une ligne dans le tableau de bord la renvoie vide (`""` partout), pas absente :
/// tout retirer enregistre une liste de lignes vides, qui ne rend pas les sections d'avant.
#[test]
#[serial]
fn removing_every_row_hides_the_legacy_sections() {
    reset_test_store();
    seed_legacy_section(0, &[("fr", "Bienvenue", "Bonjour à tous.")]);
    let removed = json!({ "show_from": "booking",
        "sections": [{ "id": "", "title": "", "body": "", "photo": "" }] });
    let (card, readiness) = guest().with_config(&removed).run(|ctx| {
        (
            render_home_card(ctx.clone()).expect("card"),
            publish_readiness(ctx).expect("readiness"),
        )
    });
    assert!(SurfaceAssertions::new(&card).contains_type("EmptyState"));
    // A removed row never blocks « Publier ».
    assert!(readiness.items.is_empty());
}
