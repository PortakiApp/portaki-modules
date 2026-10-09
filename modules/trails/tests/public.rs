//! Le bloc « Randonnées » de la page publique, rendu pour un visiteur sans séjour.

use portaki_sdk::capability;
use portaki_sdk::sdui::common::GeoPoint;
use portaki_sdk::surfaces::check_property_public_tree;
use portaki_test_utils::{MockContext, MockContextBuilder};
use serde_json::{json, Value};
use serial_test::serial;
use trails::{publish_readiness, render_host_main, render_property_public};

#[path = "../../../support/config_save.rs"]
#[allow(dead_code)]
mod config_save;

const EMISSIONS: &str = concat!(env!("OUT_DIR"), "/portaki-emissions");

/// La position réelle du logement, que seul l'hôte voit.
const HOME: (f64, f64) = (43.5600, 7.1300);
/// À une centaine de mètres : au logement pour la page publique, pas pour le plan du livret.
const NEXT_DOOR: (f64, f64) = (43.5609, 7.1302);
/// À une dizaine de kilomètres.
const FAR: (f64, f64) = (43.6280, 7.0980);

/// Des valeurs reconnaissables : aucune ne doit sortir dans l'arbre public.
const FORBIDDEN: &[&str] = &[
    "Chemin du Calvaire",
    "Saint-Jeannet",
    "visorando",
    "portaki-file:gpx",
    "43.62",
    "7.098",
    "43.560",
    "Montée courte",
];

fn trail(id: &str, title: &str, level: &str) -> Value {
    json!({
        "id": id, "title": title, "level": level,
        "duration_min": 150, "distance_km": 8, "elevation_m": 600,
        "description": "Montée courte par le sentier.",
        "address": "Saint-Jeannet", "lat": FAR.0, "lng": FAR.1,
        "link_url": "https://www.visorando.com/baou",
        "gpx_file": "portaki-file:gpx-baou",
        "photo": "portaki-file:photo-baou"
    })
}

fn config(enabled: bool, public: &[bool]) -> Value {
    let trails: Vec<Value> = public
        .iter()
        .enumerate()
        .map(|(i, _)| trail(&format!("t{i}"), &format!("Sentier {i}"), "hard"))
        .collect();
    // Le choix multiple envoie un tableau.
    let chosen: Vec<String> = public
        .iter()
        .enumerate()
        .filter(|(_, public)| **public)
        .map(|(i, _)| format!("t{i}"))
        .collect();
    json!({
        "public_enabled": enabled,
        "public_trails": chosen,
        "trails": trails
    })
}

fn visitor(config: &Value) -> MockContextBuilder {
    let bundle: serde_json::Map<String, Value> =
        serde_json::from_str(include_str!("../i18n/fr-FR.json")).expect("lot fr");
    bundle.into_iter().fold(
        MockContext::public_visitor().with_config(config),
        |builder, (key, value)| builder.with_translation(key, value.as_str().unwrap_or_default()),
    )
}

/// L'arbre rendu, et ses enfants sous la `Section`.
fn render(config: &Value) -> (Value, String) {
    let mut out = (Value::Null, String::new());
    visitor(config).run(|ctx| {
        assert!(ctx.is_public_visitor());
        let surface = render_property_public(ctx).expect("property.public");
        let tree = serde_json::to_value(&surface).expect("json");
        let root = tree["root"].clone();
        assert_eq!(root["type"], "Section");
        assert_eq!(root["title"], "i18n:public.title");
        assert_eq!(root["subtitle"], "i18n:public.eyebrow");
        let violations = check_property_public_tree(&root);
        assert!(violations.is_empty(), "{violations:?}");
        out = (root, tree.to_string());
    });
    out
}

fn children(root: &Value) -> usize {
    root["children"].as_array().map_or(0, Vec::len)
}

#[test]
#[serial]
fn the_chosen_trails_show_without_anything_that_locates_them() {
    let mut config = config(true, &[true, false, true, true]);
    config["trails"][0]["address"] = json!("Chemin du Calvaire");
    let (root, json) = render(&config);
    assert_eq!(children(&root), 1, "{json}");
    for forbidden in FORBIDDEN {
        assert!(!json.contains(forbidden), "« {forbidden} » dans {json}");
    }
    // Trois cochés sur quatre : le décoché n'y est pas.
    assert!(
        json.contains("Sentier 0") && json.contains("Sentier 3"),
        "{json}"
    );
    assert!(!json.contains("Sentier 1"), "{json}");
    // La photo, le niveau en pastille, les mesures traduites.
    assert!(json.contains("portaki-file:photo-baou"), "{json}");
    assert!(json.contains(r#""type":"Badge""#), "{json}");
    assert!(json.contains("▲▲▲ Sportif"), "{json}");
    assert!(json.contains("2 h 30 · 8 km · ↑ 600 m"), "{json}");
}

#[test]
#[serial]
fn a_disabled_block_is_an_empty_section() {
    let (root, json) = render(&config(false, &[true, true, true]));
    assert_eq!(children(&root), 0, "{json}");
    // Et la configuration vide du mock de conformité aussi.
    let (root, json) = render(&json!({}));
    assert_eq!(children(&root), 0, "{json}");
}

#[test]
#[serial]
fn fewer_than_two_chosen_is_an_empty_section() {
    let (root, json) = render(&config(true, &[true, false, false]));
    assert_eq!(children(&root), 0, "{json}");
}

#[test]
#[serial]
fn more_than_four_chosen_shows_the_first_four() {
    let (_, json) = render(&config(true, &[true; 6]));
    assert!(json.contains("Sentier 3"), "{json}");
    assert!(!json.contains("Sentier 4"), "{json}");
}

#[test]
#[serial]
fn a_start_at_the_property_says_so_and_nothing_more() {
    let mut config = config(true, &[true, true]);
    config["trails"][0]["starts_at_property"] = json!(true);
    config["trails"][0]["lat"] = json!(NEXT_DOOR.0);
    config["trails"][0]["lng"] = json!(NEXT_DOOR.1);
    let (_, json) = render(&config);
    assert_eq!(
        json.matches("i18n:public.startAtProperty").count(),
        1,
        "{json}"
    );
    assert!(!json.contains("43.5609"), "{json}");
}

/// Le formulaire hôte, qui connaît la position réelle, propose « départ au logement » sous 300 m,
/// et l'enregistrement le range sur la ligne.
#[test]
#[serial]
fn the_host_form_decides_the_start_at_the_property_and_saves_it() {
    let stored = json!({
        "trails": [
            { "id": "near", "title": "Près", "level": "easy", "lat": NEXT_DOOR.0, "lng": NEXT_DOOR.1 },
            { "id": "far", "title": "Loin", "level": "easy", "lat": FAR.0, "lng": FAR.1 },
            { "id": "kept", "title": "Gardé", "level": "easy", "lat": FAR.0, "lng": FAR.1,
              "starts_at_property": true }
        ]
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_coordinates(Some(GeoPoint::new(HOME.0, HOME.1)))
        .with_config(&stored)
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            let saved = config_save::save(EMISSIONS, &surface, &stored, "fr");
            assert_eq!(saved["trails"][0]["starts_at_property"], true);
            assert_eq!(saved["trails"][1]["starts_at_property"], false);
            // Une réponse déjà enregistrée par l'hôte n'est pas recalculée.
            assert_eq!(saved["trails"][2]["starts_at_property"], true);
            let reread: trails::ModuleConfig = serde_json::from_value(saved).expect("relue");
            assert_eq!(reread.trails[0].starts_at_property, Some(true));
        });
}

/// La carte « Page publique » : l'interrupteur du bloc et le choix multiple. Le choix est
/// déclaré `structured` (le tableau de bord envoie un vrai tableau, qu'un `text` ferait refuser),
/// sans ligne ni traduction, et se réécrit en tableau.
#[test]
#[serial]
fn the_host_form_has_a_public_page_card() {
    let field = config_save::declared_fields(EMISSIONS)
        .into_iter()
        .find(|f| f["key"] == "public_trails")
        .expect("public_trails declared");
    assert_eq!(field["type"], "structured", "{field}");
    assert!(
        field.get("item").is_none() && field.get("itemType").is_none(),
        "{field}"
    );
    let stored = config(true, &[true, false, true]);
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            let json = serde_json::to_string(&surface).unwrap();
            assert!(json.contains(r#""multi":true"#), "{json}");
            assert!(json.contains(r#""limit":4"#), "{json}");
            let args = config_save::form_args(&surface);
            assert_eq!(args["public_enabled"], true);
            // Le choix montre la sélection ; le tableau de bord renvoie la nouvelle en tableau.
            assert_eq!(args["public_trails"], r#"["t0","t2"]"#);
            let mut saved = config_save::save(EMISSIONS, &surface, &stored, "fr");
            saved["public_trails"] = json!(["t2", "t0"]);
            let reread: trails::ModuleConfig = serde_json::from_value(saved).expect("relue");
            assert!(reread.public_enabled);
            assert_eq!(reread.public_trails, ["t2", "t0"]);
            let written = serde_json::to_value(&reread).unwrap();
            assert_eq!(written["public_trails"], json!(["t2", "t0"]));
        });
    // Rien à choisir : le choix est rendu quand même, pour que la clé parte.
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .run(|ctx| {
            let args = config_save::form_args(&render_host_main(ctx).expect("host main"));
            assert_eq!(args["public_trails"], "[]");
        });
}

/// Le choix se lit en tableau, ou comme les brouillons d'avant : en JSON texte ou en liste à
/// virgules ; un id qui n'est plus un itinéraire affichable ne compte pas.
#[test]
#[serial]
fn the_choice_reads_leniently_and_ignores_gone_trails() {
    for chosen in [
        json!(["t0", "gone", "t1"]),
        json!("t0, gone ,t1"),
        json!(r#"["t0","gone","t1","t0"]"#),
    ] {
        let mut config = config(true, &[false, false]);
        config["public_trails"] = chosen;
        let (_, json) = render(&config);
        assert!(
            json.contains("Sentier 0") && json.contains("Sentier 1"),
            "{json}"
        );
    }
    let mut config = config(true, &[false, false]);
    config["public_trails"] = json!(["t0", "gone"]);
    let (root, json) = render(&config);
    assert_eq!(children(&root), 0, "{json}");
}

/// Hors bornes, la publication est recommandée, jamais bloquée.
#[test]
#[serial]
fn an_out_of_bounds_choice_warns_without_blocking() {
    for (config, warned) in [
        (config(true, &[true]), true),
        (config(true, &[true; 5]), true),
        (config(true, &[true, true]), false),
        (config(false, &[true]), false),
    ] {
        MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&config)
            .run(|ctx| {
                let readiness = publish_readiness(ctx).expect("readiness");
                let check = readiness
                    .items
                    .iter()
                    .find(|item| item.id == "config.public_enabled");
                assert_eq!(check.is_some(), warned, "{config}");
                if let Some(check) = check {
                    assert_eq!(
                        serde_json::to_value(check.level).unwrap(),
                        json!("recommended")
                    );
                }
            });
    }
}
