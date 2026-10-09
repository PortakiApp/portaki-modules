//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use portaki_sdk::sdui::common::GeoPoint;
use portaki_test_utils::{MockContext, Property, SurfaceAssertions};
use serde_json::{json, Value};
use serial_test::serial;
use trails::{
    map_markers, publish_readiness, render_explore_detail, render_explore_item, render_home_card,
    render_host_main, MAX_TRAILS,
};

#[path = "../../../support/config_form.rs"]
mod config_form;
#[path = "../../../support/config_save.rs"]
mod config_save;

const EMISSIONS: &str = concat!(env!("OUT_DIR"), "/portaki-emissions");

/// Le logement du Cap d'Antibes.
const HOME: (f64, f64) = (43.5600, 7.1300);
/// À une cinquantaine de mètres : le départ est devant la porte.
const DOORSTEP: (f64, f64) = (43.5604, 7.1302);
/// À quelques centaines de mètres : on y va à pied, sans itinéraire.
const NEARBY: (f64, f64) = (43.5635, 7.1310);
/// À une dizaine de kilomètres : il faut y aller.
const FAR: (f64, f64) = (43.6280, 7.0980);

fn sample_config() -> Value {
    json!({
        "trails": [
            { "id": "baou", "title": "Baou de Saint-Jeannet", "level": "hard", "shape": "round_trip",
              "duration_min": 240, "distance_km": 8, "elevation_m": 600,
              "address": "Saint-Jeannet", "lat": FAR.0, "lng": FAR.1,
              "link_url": "https://www.visorando.com/baou" },
            { "id": "garoupe", "title": "Phare de la Garoupe", "level": "easy", "shape": "round_trip",
              "duration_min": 60, "distance_km": 2.6, "elevation_m": 80,
              "address": "Chemin du Calvaire", "lat": DOORSTEP.0, "lng": DOORSTEP.1 },
            { "id": "littoral", "title": "Sentier du littoral", "level": "easy", "shape": "loop",
              "duration_min": 150, "distance_km": 8, "elevation_m": 60 },
            { "id": "fort", "title": "Fort Carré", "level": "moderate", "shape": "loop",
              "duration_min": 75, "distance_km": 3.5,
              "address": "Port Vauban", "lat": NEARBY.0, "lng": NEARBY.1 }
        ],
        "commune_url": "https://www.antibesjuanlespins.com/randonnees"
    })
}

/// Le lot français du module, chargé dans le contexte.
///
/// Sans lui, le mock rend chaque clé telle quelle : « ▲ Facile · 2 » devient `guest.level.count`,
/// et deux pastilles de niveaux différents deviennent le même texte. Les assertions porteraient
/// alors sur rien.
fn with_fr(
    builder: portaki_test_utils::MockContextBuilder,
) -> portaki_test_utils::MockContextBuilder {
    let bundle: serde_json::Map<String, Value> =
        serde_json::from_str(include_str!("../i18n/fr-FR.json")).expect("lot fr");
    bundle.into_iter().fold(builder, |builder, (key, value)| {
        builder.with_translation(key, value.as_str().unwrap_or_default())
    })
}

fn guest() -> portaki_test_utils::MockContextBuilder {
    with_fr(
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_property(Property::default())
            .with_coordinates(Some(GeoPoint::new(HOME.0, HOME.1))),
    )
}

fn tree(surface: &portaki_sdk::sdui::surface::Surface) -> String {
    serde_json::to_string(surface).expect("surface json")
}

/// La carte : une pastille par niveau présent, et deux itinéraires, pas plus.
#[test]
#[serial]
fn the_home_card_counts_the_levels_and_shows_two_trails() {
    guest().with_config(&sample_config()).run(|ctx| {
        let surface = render_home_card(ctx).expect("home card");
        let json = tree(&surface);
        assert!(SurfaceAssertions::new(&surface).contains_type("Badge"));
        assert_eq!(json.matches(r#""type":"ListItem""#).count(), 2, "{json}");
        // L'ordre des pastilles vient des niveaux, pas de la saisie : Facile avant Sportif, alors
        // que l'hôte a saisi le baou en premier.
        let easy = json.find("▲ Facile · 2").expect("pastille facile");
        let moderate = json.find("▲▲ Moyen · 1").expect("pastille moyen");
        let hard = json.find("▲▲▲ Sportif · 1").expect("pastille sportif");
        assert!(easy < moderate && moderate < hard, "{json}");
        assert!(json.contains(r#""presentation":"fullscreen""#), "{json}");
    });
}

/// Rien de complet : l'état vide, sur les deux surfaces.
#[test]
#[serial]
fn nothing_complete_shows_the_empty_state() {
    for config in [json!({}), json!({ "trails": [{ "title": "Sans niveau" }] })] {
        guest().with_config(&config).run(|ctx| {
            for surface in [
                render_home_card(ctx.clone()).expect("home card"),
                render_explore_detail(ctx.clone()).expect("list"),
            ] {
                assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
            }
        });
    }
}

/// La liste : une section par niveau, dans l'ordre croissant, et le lien de la commune en bas.
#[test]
#[serial]
fn the_list_groups_by_level_in_order() {
    guest().with_config(&sample_config()).run(|ctx| {
        let json = tree(&render_explore_detail(ctx).expect("list"));
        let easy = json.find("▲ Facile · 2").expect("section facile");
        let hard = json.find("▲▲▲ Sportif · 1").expect("section sportif");
        assert!(
            easy < hard,
            "l'ordre des niveaux ne suit pas la saisie : {json}"
        );
        // Quatre itinéraires, trois sections.
        assert_eq!(json.matches(r#""type":"Card""#).count(), 3, "{json}");
        assert_eq!(json.matches(r#""type":"ListItem""#).count(), 4, "{json}");
        // La forme s'affiche dans la liste, où l'on compare.
        assert!(json.contains("Boucle"), "{json}");
        assert!(json.contains("antibesjuanlespins.com"), "{json}");
    });
}

/// Un lien de commune qui n'est pas une adresse https ne s'affiche pas.
#[test]
#[serial]
fn a_commune_link_that_is_not_https_is_dropped() {
    let config = json!({
        "trails": [{ "title": "A", "level": "easy" }],
        "commune_url": "javascript:alert(1)"
    });
    guest().with_config(&config).run(|ctx| {
        let json = tree(&render_explore_detail(ctx).expect("list"));
        assert!(!json.contains(r#""type":"Link""#), "{json}");
    });
}

/// La fiche : les quatre tuiles, le départ, le plan, et les deux boutons.
#[test]
#[serial]
fn a_far_trail_offers_the_route_after_the_trace() {
    guest().with_config(&sample_config()).run(|ctx| {
        let mut item = ctx.clone();
        item.input = json!({ "trailId": "baou" });
        let json = tree(&render_explore_item(item).expect("item"));
        assert_eq!(json.matches(r#""layout":"tile""#).count(), 4, "{json}");
        assert!(json.contains("Départ : Saint-Jeannet"), "{json}");
        // Deux repères : le logement et un départ qui n'est pas lui.
        assert_eq!(json.matches(r#""kind":"p"#).count(), 2, "{json}");
        let trace = json.find("guest.openTrace").expect("bouton trace");
        let route = json.find("guest.routeToStart").expect("bouton itinéraire");
        assert!(trace < route, "{json}");
        // L'itinéraire est second, donc en contour.
        assert!(json.contains(r#""variant":"outline""#), "{json}");
    });
}

/// Un départ devant le logement : pas de second repère, pas de bouton d'itinéraire.
#[test]
#[serial]
fn a_trail_from_the_doorstep_needs_no_directions() {
    guest().with_config(&sample_config()).run(|ctx| {
        let mut item = ctx.clone();
        item.input = json!({ "trailId": "garoupe" });
        let json = tree(&render_explore_item(item).expect("item"));
        assert!(json.contains("Départ : devant le logement"), "{json}");
        assert_eq!(json.matches(r#""kind":"p"#).count(), 1, "{json}");
        assert!(!json.contains("guest.routeToStart"), "{json}");
        // Ni fiche tierce ni itinéraire : aucun bouton, et pas de barre vide.
        assert!(!json.contains(r#""type":"Button""#), "{json}");
    });
}

/// Un départ à quelques centaines de mètres : la distance à pied, et pas de bouton d'itinéraire.
#[test]
#[serial]
fn a_nearby_start_shows_the_walking_distance() {
    guest().with_config(&sample_config()).run(|ctx| {
        let mut item = ctx.clone();
        item.input = json!({ "trailId": "fort" });
        let json = tree(&render_explore_item(item).expect("item"));
        // Arrondi à 50 m : la ligne droite ne mérite pas le mètre près.
        assert!(json.contains("Départ : Port Vauban · 4"), "{json}");
        assert!(json.contains(" m"), "{json}");
        assert!(!json.contains("guest.routeToStart"), "{json}");
    });
}

/// Sans position du départ, pas de plan et pas d'itinéraire — mais la fiche se lit.
#[test]
#[serial]
fn a_trail_without_a_position_still_renders() {
    guest().with_config(&sample_config()).run(|ctx| {
        let mut item = ctx.clone();
        item.input = json!({ "trailId": "littoral" });
        let surface = render_explore_item(item).expect("item");
        let json = tree(&surface);
        // Le plan garde le logement seul : un repère vaut mieux que rien quand on sait où l'on est.
        assert!(json.contains(r#""kind":"property""#), "{json}");
        assert!(!json.contains("guest.map.start"), "{json}");
        assert!(!json.contains("guest.routeToStart"), "{json}");
        assert!(SurfaceAssertions::new(&surface).contains_type("Grid"));
    });
}

/// Un identifiant qui n'existe plus ne rend pas une fiche vide muette.
#[test]
#[serial]
fn an_unknown_trail_id_says_so() {
    guest().with_config(&sample_config()).run(|ctx| {
        let mut item = ctx.clone();
        item.input = json!({ "trailId": "mont-blanc" });
        let surface = render_explore_item(item).expect("item");
        assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
        assert!(
            tree(&surface).contains("guest.item.notFound"),
            "introuvable"
        );
    });
}

/// Les tuiles que l'hôte n'a pas renseignées ne laissent pas de trou.
#[test]
#[serial]
fn a_trail_without_measures_drops_its_tiles() {
    let config = json!({
        "trails": [{ "id": "x", "title": "Tour du village", "level": "easy", "duration_min": 30 }]
    });
    guest().with_config(&config).run(|ctx| {
        let mut item = ctx.clone();
        item.input = json!({ "trailId": "x" });
        let json = tree(&render_explore_item(item).expect("item"));
        assert_eq!(json.matches(r#""layout":"tile""#).count(), 1, "{json}");
        assert!(!json.contains("guest.tile.distance"), "{json}");
    });
}

/// Un aller-retour enregistré `out_and_back` par l'ancien pré-remplissage GPX garde sa tuile.
#[test]
#[serial]
fn a_stored_out_and_back_keeps_its_type_tile() {
    let config = json!({
        "trails": [{ "id": "x", "title": "Baou", "level": "hard", "shape": "out_and_back",
                     "gpx_file": "portaki-file:6f1c1d2e-3a4b-4c5d-8e9f-0a1b2c3d4e5f" }]
    });
    guest().with_config(&config).run(|ctx| {
        let mut item = ctx.clone();
        item.input = json!({ "trailId": "x" });
        let json = tree(&render_explore_item(item).expect("item"));
        assert!(json.contains("guest.tile.shape"), "{json}");
        assert!(json.contains("Aller-retour"), "{json}");
    });
}

/// Le formulaire hôte dessine ce que la configuration déclare, et une ligne par itinéraire stocké.
#[test]
#[serial]
fn the_host_form_draws_the_rows_the_host_has() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            config_form::assert_form_matches_config(EMISSIONS, &surface, &[]);
            let json = tree(&surface);
            assert!(json.contains("trails.3.elevation_m"), "{json}");
            // Quatre itinéraires stockés : pas de cinquième ligne vide.
            assert!(!json.contains("trails.4.title"), "{json}");
            assert!(json.contains(r#""value":"hard""#), "{json}");
            assert!(json.contains("Phare de la Garoupe"), "{json}");
        });
}

/// Rien de stocké : une ligne, et « Ajouter » en demande une de plus — bornée à douze.
#[test]
#[serial]
fn the_form_grows_one_row_at_a_time_up_to_the_bound() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .run(|ctx| {
            let json = tree(&render_host_main(ctx).expect("host main"));
            assert!(json.contains("trails.0.title"), "{json}");
            assert!(!json.contains("trails.1.title"), "{json}");
            assert!(json.contains(r#""trails_count":2"#), "{json}");
        });

    let rows: Vec<Value> = (0..MAX_TRAILS)
        .map(|i| json!({ "title": format!("Sentier {i}"), "level": "easy" }))
        .collect();
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "trails": rows }))
        .run(|ctx| {
            let json = tree(&render_host_main(ctx).expect("host main"));
            let last = MAX_TRAILS - 1;
            assert!(json.contains(&format!("trails.{last}.title")), "{json}");
            assert!(
                !json.contains(&format!("trails.{}.title", last + 1)),
                "{json}"
            );
            assert!(
                json.contains(&format!(r#""trails_count":{MAX_TRAILS}"#)),
                "{json}"
            );
        });
}

/// L'enregistrement rend les deux langues : le formulaire envoie celle qu'il édite, l'autre reste.
#[test]
#[serial]
fn saving_in_english_keeps_the_french_text() {
    let stored = json!({
        "trails": [{
            "id": "garoupe",
            "level": "easy",
            "shape": "round_trip",
            "duration_min": 60,
            "distance_km": 2.6,
            "title": { "fr": "Phare de la Garoupe", "en": "La Garoupe lighthouse" },
            "description": { "fr": "Montée courte.", "en": "A short climb." }
        }],
        "commune_url": "https://www.antibesjuanlespins.com"
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|mut ctx| {
            ctx.locale = "en-US".into();
            let surface = render_host_main(ctx).expect("host main");
            // L'inventaire des textes traduits : ce qu'un hôte qui écrit en anglais garde en
            // français. Une ligne qui disparaît d'ici est une traduction perdue.
            assert_eq!(
                config_save::localized_paths(EMISSIONS),
                ["trails.description", "trails.title"]
            );
            let saved = config_save::save(EMISSIONS, &surface, &stored, "en");
            assert_eq!(saved["trails"][0]["title"]["fr"], "Phare de la Garoupe");
            assert_eq!(saved["trails"][0]["title"]["en"], "La Garoupe lighthouse");
            // Les mesures et les listes figées traversent l'enregistrement.
            assert_eq!(saved["trails"][0]["level"], "easy");
            assert_eq!(saved["trails"][0]["duration_min"], 60.0);
            assert_eq!(saved["trails"][0]["distance_km"], 2.6);
            assert_eq!(saved["commune_url"], stored["commune_url"]);
            // Et surtout : ce que l'enregistrement a écrit se relit. Les mesures reviennent en
            // flottants (`60.0`), ce qu'un entier refusait — la configuration entière devenait
            // illisible et le module rendait son état d'erreur.
            let reread: trails::ModuleConfig =
                serde_json::from_value(saved).expect("la config relue après enregistrement");
            assert_eq!(reread.trails[0].duration(), Some(60));
        });
}

/// La porte de publication : un itinéraire complet suffit ; un niveau manquant bloque sur sa
/// ligne, un départ sans position et des mesures absentes se recommandent.
#[test]
#[serial]
fn publication_needs_one_complete_trail() {
    let check = |config: Value| {
        MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&config)
            .run(|ctx| publish_readiness(ctx).expect("readiness"))
    };

    let empty = check(json!({}));
    assert!(!empty.items[0].ok);
    assert_eq!(empty.items.len(), 1);

    let ready = check(json!({
        "trails": [{ "title": "A", "level": "easy", "duration_min": 60, "distance_km": 3,
                      "lat": 43.56, "lng": 7.12 }]
    }));
    assert!(ready.items[0].ok);
    assert_eq!(ready.items.len(), 1);

    let half = check(json!({
        "trails": [
            { "title": "A", "level": "easy" },
            { "title": "B" }
        ]
    }));
    assert!(half.items[0].ok);
    let ids: Vec<&str> = half.items.iter().map(|item| item.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "trails",
            "config.trails.1.level",
            "config.trails.0.lat",
            "measures"
        ]
    );
}

/// La carte du livret ne reçoit que les départs situés.
#[test]
#[serial]
fn only_positioned_trails_reach_the_map() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let markers = map_markers(ctx).expect("markers").markers;
            let ids: Vec<&str> = markers.iter().map(|m| m.id.as_str()).collect();
            // `littoral` n'a pas de position : il ne pose pas de repère.
            assert_eq!(ids, ["trail-baou", "trail-garoupe", "trail-fort"]);
            assert_eq!(markers[0].category.as_deref(), Some("hiking"));
            assert_eq!(markers[0].label.as_deref(), Some("Baou de Saint-Jeannet"));
        });
}

/// Un seul niveau : pas de pastilles (§2.23).
///
/// Une pastille unique ne trie rien — elle répète ce que chaque rangée dit déjà en fin de ligne,
/// et annonce un choix qui n'existe pas. Les itinéraires, eux, restent là.
#[test]
#[serial]
fn one_level_wears_no_pill() {
    guest()
        .with_config(&json!({
            "trails": [
                { "id": "a", "title": "Phare", "level": "easy", "shape": "loop",
                  "duration_min": 60, "distance_km": 2.6, "elevation_m": 80 },
                { "id": "b", "title": "Littoral", "level": "easy", "shape": "loop",
                  "duration_min": 150, "distance_km": 8, "elevation_m": 60 }
            ]
        }))
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("home card");
            let json = tree(&surface);
            assert!(
                !SurfaceAssertions::new(&surface).contains_type("Badge"),
                "{json}"
            );
            assert_eq!(json.matches(r#""type":"ListItem""#).count(), 2, "{json}");
            // Le niveau reste en fin de rangée : c'est là qu'il se lit.
            assert!(json.contains("▲ Facile"), "{json}");
        });
}
