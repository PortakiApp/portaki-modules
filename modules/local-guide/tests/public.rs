//! Le bloc « Adresses » de la page publique, rendu pour un visiteur sans séjour.

use local_guide::{publish_readiness, render_host_main, render_property_public};
use portaki_sdk::capability;
use portaki_sdk::sdui::common::GeoPoint;
use portaki_sdk::surfaces::check_property_public_tree;
use portaki_test_utils::{MockContext, MockContextBuilder};
use serde_json::{json, Value};
use serial_test::serial;

#[path = "../../../support/config_save.rs"]
#[allow(dead_code)]
mod config_save;

const EMISSIONS: &str = concat!(env!("OUT_DIR"), "/portaki-emissions");

/// Le centre flouté du logement, tel que la plateforme l'envoie au visiteur public.
const CENTRE: (f64, f64) = (43.5600, 7.1300);
/// À environ 800 m : à pied.
const NEAR: (f64, f64) = (43.5672, 7.1300);
/// À environ 8 km : en voiture.
const FAR: (f64, f64) = (43.6280, 7.0980);

/// Des valeurs reconnaissables : aucune ne doit sortir dans l'arbre public.
const FORBIDDEN: &[&str] = &[
    "10 % sur le menu",
    "-10 %",
    "€€€",
    "+33 4 93",
    "12:00 – 14:30",
    "Fermé le lundi",
    "Parking du port",
    "Quai des pêcheurs",
    "Demandez Marco",
    "Terrasse sur le port",
    "400 m à pied",
    "https://chez-marco.example",
    "43.5672",
    "43.628",
];

fn spot(index: usize, at: (f64, f64)) -> Value {
    json!({
        "id": format!("s{index}"),
        "title": format!("Adresse {index}"),
        "category": "Restaurant",
        "distance": "400 m à pied",
        "tag": "-10 %",
        "detail": "Terrasse sur le port.",
        "note": { "fr": "Demandez Marco de ma part." },
        "address": "Quai des pêcheurs, Antibes",
        "lat": at.0, "lng": at.1,
        "perk": "10 % sur le menu du soir.",
        "price": "€€€",
        "hours": "12:00 – 14:30",
        "closed_day": "mon",
        "opening": "Fermé le lundi",
        "parking": "Parking du port",
        "phone": "+33 4 93 00 00 01",
        "url": "https://chez-marco.example",
        "photos": [format!("portaki-file:photo-{index}")]
    })
}

fn config(enabled: bool, public: &[bool]) -> Value {
    let spots: Vec<Value> = public
        .iter()
        .enumerate()
        .map(|(i, _)| spot(i, if i == 0 { FAR } else { NEAR }))
        .collect();
    // Le choix multiple envoie un tableau.
    let chosen: Vec<String> = public
        .iter()
        .enumerate()
        .filter(|(_, public)| **public)
        .map(|(i, _)| format!("s{i}"))
        .collect();
    json!({
        "public_enabled": enabled,
        "public_spots": chosen,
        "spots": spots
    })
}

fn visitor(config: &Value) -> MockContextBuilder {
    let bundle: serde_json::Map<String, Value> =
        serde_json::from_str(include_str!("../i18n/fr-FR.json")).expect("lot fr");
    bundle.into_iter().fold(
        MockContext::public_visitor()
            .with_coordinates(Some(GeoPoint::new(CENTRE.0, CENTRE.1)))
            .with_config(config),
        |builder, (key, value)| builder.with_translation(key, value.as_str().unwrap_or_default()),
    )
}

/// L'arbre rendu : la `Section` racine, et son texte.
fn render(config: &Value) -> (Value, String) {
    let mut out = (Value::Null, String::new());
    visitor(config).run(|ctx| {
        assert!(ctx.is_public_visitor());
        let surface = render_property_public(ctx).expect("property.public");
        let root = serde_json::to_value(&surface).expect("json")["root"].clone();
        assert_eq!(root["type"], "Section");
        assert_eq!(root["title"], "i18n:public.title");
        assert_eq!(root["subtitle"], "i18n:public.eyebrow");
        let violations = check_property_public_tree(&root);
        assert!(violations.is_empty(), "{violations:?}");
        let json = root.to_string();
        out = (root, json);
    });
    out
}

fn children(root: &Value) -> usize {
    root["children"].as_array().map_or(0, Vec::len)
}

#[test]
#[serial]
fn the_chosen_places_show_without_perk_price_phone_or_address() {
    let (root, json) = render(&config(true, &[true, true, false, true]));
    // La grille et la phrase du livret.
    assert_eq!(children(&root), 2, "{json}");
    for forbidden in FORBIDDEN {
        assert!(!json.contains(forbidden), "« {forbidden} » dans {json}");
    }
    assert!(
        json.contains("Adresse 0") && json.contains("Adresse 3"),
        "{json}"
    );
    assert!(!json.contains("Adresse 2"), "{json}");
    assert!(json.contains("portaki-file:photo-0"), "{json}");
    assert!(json.contains("Restaurant"), "{json}");
    assert!(json.contains("i18n:public.more"), "{json}");
}

/// Depuis le centre flouté, la distance s'annonce approximative, à pied ou en voiture.
#[test]
#[serial]
fn the_distance_is_approximate() {
    let (_, json) = render(&config(true, &[true, true, true]));
    // ~800 m à 5 km/h = 10 min ; ~8 km à 30 km/h = 17 min → 20.
    assert!(json.contains("~10 min à pied"), "{json}");
    assert!(json.contains("~20 min en voiture"), "{json}");
}

/// Sans position, ni pour l'adresse ni pour le logement : pas de distance, et jamais le texte
/// libre que l'hôte a tapé à la place.
#[test]
#[serial]
fn no_position_means_no_distance() {
    let mut config = config(true, &[true, true, true]);
    for spot in config["spots"].as_array_mut().unwrap() {
        spot["lat"] = Value::Null;
        spot["lng"] = Value::Null;
    }
    let (_, json) = render(&config);
    assert!(
        !json.contains("min à pied") && !json.contains("min en voiture"),
        "{json}"
    );
    assert!(!json.contains("400 m"), "{json}");
}

#[test]
#[serial]
fn a_disabled_block_is_an_empty_section() {
    let (root, json) = render(&config(false, &[true; 4]));
    assert_eq!(children(&root), 0, "{json}");
    let (root, json) = render(&json!({}));
    assert_eq!(children(&root), 0, "{json}");
}

#[test]
#[serial]
fn fewer_than_three_chosen_is_an_empty_section() {
    let (root, json) = render(&config(true, &[true, true, false, false]));
    assert_eq!(children(&root), 0, "{json}");
    // Une adresse cochée mais sans nom n'est pas publiée : elle ne compte pas.
    let mut config = config(true, &[true, true, true]);
    config["spots"][2]["title"] = json!("");
    let (root, json) = render(&config);
    assert_eq!(children(&root), 0, "{json}");
}

#[test]
#[serial]
fn more_than_six_chosen_shows_the_first_six() {
    let (_, json) = render(&config(true, &[true; 8]));
    assert!(json.contains("Adresse 5"), "{json}");
    assert!(!json.contains("Adresse 6"), "{json}");
}

/// La carte « Page publique » : l'interrupteur du bloc et le choix multiple. Le choix est
/// déclaré `structured` (le tableau de bord envoie un vrai tableau, qu'un `text` ferait refuser),
/// sans ligne ni traduction, et se réécrit en tableau.
#[test]
#[serial]
fn the_host_form_has_a_public_page_card() {
    let field = config_save::declared_fields(EMISSIONS)
        .into_iter()
        .find(|f| f["key"] == "public_spots")
        .expect("public_spots declared");
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
            assert!(json.contains(r#""limit":6"#), "{json}");
            let args = config_save::form_args(&surface);
            assert_eq!(args["public_enabled"], true);
            // Le choix montre la sélection ; le tableau de bord renvoie la nouvelle en tableau.
            assert_eq!(args["public_spots"], r#"["s0","s2"]"#);
            let mut saved = config_save::save(EMISSIONS, &surface, &stored, "fr");
            saved["public_spots"] = json!(["s2", "s0"]);
            let reread: local_guide::ModuleConfig = serde_json::from_value(saved).expect("relue");
            assert!(reread.public_enabled);
            assert_eq!(reread.public_spots, ["s2", "s0"]);
            let written = serde_json::to_value(&reread).unwrap();
            assert_eq!(written["public_spots"], json!(["s2", "s0"]));
        });
    // Rien à choisir : le choix est rendu quand même, pour que la clé parte.
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .run(|ctx| {
            let args = config_save::form_args(&render_host_main(ctx).expect("host main"));
            assert_eq!(args["public_spots"], "[]");
        });
}

/// Le choix se lit en tableau, ou comme les brouillons d'avant : en JSON texte ou en liste à
/// virgules ; un id qui n'est plus une adresse publiée ne compte pas.
#[test]
#[serial]
fn the_choice_reads_leniently_and_ignores_gone_spots() {
    for chosen in [
        json!(["s0", "gone", "s1", "s2"]),
        json!("s0, gone ,s1,s2"),
        json!(r#"["s0","gone","s1","s2","s0"]"#),
    ] {
        let mut config = config(true, &[false, false, false]);
        config["public_spots"] = chosen;
        let (root, json) = render(&config);
        assert_eq!(children(&root), 2, "{json}");
    }
    let mut config = config(true, &[false, false, false]);
    config["public_spots"] = json!(["s0", "s1", "gone"]);
    let (root, json) = render(&config);
    assert_eq!(children(&root), 0, "{json}");
}

/// Hors bornes, la publication est recommandée, jamais bloquée.
#[test]
#[serial]
fn an_out_of_bounds_choice_warns_without_blocking() {
    for (config, warned) in [
        (config(true, &[true, true]), true),
        (config(true, &[true; 7]), true),
        (config(true, &[true; 3]), false),
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

/// Une catégorie manquante avertit sans bloquer ; un lien sans https bloque toujours.
#[test]
#[serial]
fn a_missing_category_warns_without_blocking() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "spots": [{ "title": "Le Bacon", "lat": 43.55, "lng": 7.01, "url": "http://bacon.fr" }]
        }))
        .run(|ctx| {
            let readiness = publish_readiness(ctx).expect("readiness");
            let level = |id: &str| {
                let item = readiness.items.iter().find(|i| i.id == id).expect(id);
                serde_json::to_value(item.level).unwrap()
            };
            assert_eq!(level("config.spots.0.category"), json!("recommended"));
            assert_eq!(level("config.spots.0.url"), json!("required"));
            let hint = &readiness
                .items
                .iter()
                .find(|i| i.id == "config.spots.0.category")
                .unwrap()
                .hint;
            assert_eq!(hint.get("fr"), "Choisissez une catégorie.");
        });
}

/// Un téléphone d'activité sans indicatif avertit sans bloquer : des activités saisies avant la
/// règle en ont un. Celui d'une adresse reste bloquant, comme avant.
#[test]
fn an_activity_phone_warns_without_blocking() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "spots": [{ "title": "Le Bacon", "category": "restaurant", "lat": 43.55, "lng": 7.01,
                        "phone": "04 93 61 50 02" }],
            "host_activities": [{ "title": "Balade", "phone": "06 12 34 56 78" }]
        }))
        .run(|ctx| {
            let readiness = publish_readiness(ctx).expect("readiness");
            let level = |id: &str| {
                let item = readiness.items.iter().find(|i| i.id == id).expect(id);
                serde_json::to_value(item.level).unwrap()
            };
            assert_eq!(
                level("config.host_activities.0.phone"),
                json!("recommended")
            );
            assert_eq!(level("config.spots.0.phone"), json!("required"));
        });
}
