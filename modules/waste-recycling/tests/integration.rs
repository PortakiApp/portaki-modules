//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use portaki_sdk::context::StayContext;
use portaki_sdk::host::module::ModuleStatus;
use portaki_sdk::prelude::{DateTime, Utc};
use portaki_test_utils::{MockContext, SurfaceAssertions};
use serde_json::{json, Value};

use waste_recycling::{
    publish_readiness, render_explore_detail, render_home_card, render_host_main,
};

#[path = "../../../support/config_form.rs"]
mod config_form;
#[path = "../../../support/config_save.rs"]
mod config_save;

const EMISSIONS: &str = concat!(env!("OUT_DIR"), "/portaki-emissions");

fn sample_config() -> Value {
    json!({
        "bins": [
            {
                "id": "yellow",
                "title": {"fr": "Bac jaune", "en": "Yellow bin"},
                "items": {"fr": "Emballages, plastique", "en": "Packaging, plastic"},
                "color": "#f4c020"
            },
            {
                "id": "green",
                "title": {"fr": "Bac vert", "en": "Green bin"},
                "items": {"fr": "Verre", "en": "Glass"},
                "color": "#3a8a4d"
            }
        ],
        "collection_schedule": "Mardi & vendredi matin"
    })
}

#[test]
#[serial]
fn home_card_renders_empty_without_config() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("home card");
            assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
        });
}

#[test]
#[serial]
fn home_card_renders_bins_with_config() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("home card");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ColorDotItem"));
            assert!(SurfaceAssertions::new(&surface).contains_type("InfoBanner"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("bottomSheet"));
            assert!(json.contains("explore.detail"));
        });
}

/// The stored hex of an older config becomes a named swatch on the wire — no CSS color leaves
/// the module, the booklet picks the yellow.
#[test]
#[serial]
fn bins_render_as_named_swatches() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let json = serde_json::to_string(&render_home_card(ctx).expect("home card"))
                .expect("surface json");
            assert!(json.contains(r#""swatch":"yellow""#), "{json}");
            assert!(json.contains(r#""swatch":"green""#), "{json}");
            assert!(!json.contains("#f4c020"), "{json}");
        });
}

#[test]
#[serial]
fn detail_renders_enriched_bins() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("detail");
            assert!(SurfaceAssertions::new(&surface).contains_type("Stack"));
            assert!(SurfaceAssertions::new(&surface).contains_type("ColorDotItem"));
        });
}

#[test]
#[serial]
fn the_host_form_sends_the_declared_keys() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "bins": [
                { "id": "yellow", "title": { "fr": "Bac jaune" }, "items": { "fr": "Plastique\nCarton" }, "color": "#f4c020" },
                { "title": "Bac vert", "items": "Verre", "color": "green" }
            ],
            "collection_schedule": "Mardi"
        }))
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            config_form::assert_form_matches_config(
                concat!(env!("OUT_DIR"), "/portaki-emissions"),
                &surface,
                &[],
            );
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains(r"Plastique\nCarton"), "{json}");
            assert!(json.contains("Bac vert"));
            assert!(json.contains(r#""value":"yellow""#), "{json}");
        });
}

/// Mercredi 30 septembre 2026, à midi passé au logement.
fn wednesday() -> DateTime<Utc> {
    at("2026-09-30T10:00:00Z")
}

fn at(rfc3339: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(rfc3339)
        .expect("une date")
        .with_timezone(&Utc)
}

/// Un contexte voyageur avec l'horloge figée et les libellés de collecte chargés.
fn guest_at(now: DateTime<Utc>) -> portaki_test_utils::MockContextBuilder {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_now(now)
        .with_translation("guest.collection.next", "Prochaine collecte {day}")
}

fn banner(surface: &portaki_sdk::sdui::surface::Surface) -> Value {
    fn find(node: &Value) -> Option<Value> {
        if node.get("type") == Some(&Value::from("InfoBanner")) {
            return Some(node.clone());
        }
        match node {
            Value::Object(object) => object.values().find_map(find),
            Value::Array(items) => items.iter().find_map(find),
            _ => None,
        }
    }
    find(&serde_json::to_value(surface).expect("surface json")).expect("un bandeau")
}

/// Aucun jour coché : le bandeau de l'hôte, mot pour mot, comme avant (§2.7).
///
/// C'est la garantie que la nouveauté n'a rien cassé chez les hôtes qui ont déjà écrit leur phrase :
/// personne ne perd ce qu'il a saisi, personne n'est forcé à ressaisir.
#[test]
#[serial]
fn the_prose_stays_when_no_day_is_ticked() {
    guest_at(wednesday())
        .with_config(&sample_config())
        .run(|ctx| {
            let found = banner(&render_home_card(ctx).expect("home card"));
            assert_eq!(found["title"], "i18n:guest.collection.title");
            assert_eq!(found["message"], "Mardi & vendredi matin");
        });
}

/// Des jours cochés : « Prochaine collecte vendredi », et la phrase garde sa place en second.
#[test]
#[serial]
fn a_ticked_day_becomes_the_next_collection() {
    let mut config = sample_config();
    config["collects_tue"] = json!(true);
    config["collects_fri"] = json!(true);
    guest_at(wednesday()).with_config(&config).run(|ctx| {
        let found = banner(&render_home_card(ctx).expect("home card"));
        assert_eq!(found["title"], "Prochaine collecte vendredi");
        // La phrase de l'hôte dit ce qu'une case ne dit pas : elle reste sous le jour calculé.
        assert_eq!(found["message"], "Mardi & vendredi matin");
    });
}

/// Le jour même, le bandeau ne renvoie pas à la semaine prochaine.
#[test]
#[serial]
fn a_collection_today_is_said_as_today() {
    let mut config = sample_config();
    config["collects_wed"] = json!(true);
    guest_at(wednesday()).with_config(&config).run(|ctx| {
        let found = banner(&render_home_card(ctx).expect("home card"));
        assert_eq!(found["title"], "i18n:guest.collection.today");
    });
}

/// « Collecte le jour du départ → le bandeau le dit » (§2.7).
#[test]
#[serial]
fn a_collection_on_the_departure_day_is_said() {
    let mut config = sample_config();
    config["collects_fri"] = json!(true);
    guest_at(wednesday())
        .with_config(&config)
        .with_stay(StayContext {
            checkout_at: Some(at("2026-10-02T09:00:00Z")),
            ..StayContext::default()
        })
        .run(|ctx| {
            let found = banner(&render_home_card(ctx).expect("home card"));
            assert_eq!(found["title"], "Prochaine collecte vendredi");
            assert_eq!(found["message"], "i18n:guest.collection.departureDay");
        });
}

/// « Départ la veille d'une collecte → consigne de sortie mise en avant » (§2.7) : le voyageur ne
/// sera plus là vendredi matin, et doit donc sortir le bac avant de fermer la porte.
#[test]
#[serial]
fn leaving_the_day_before_pushes_the_takeout_note_up() {
    let mut config = sample_config();
    config["collects_fri"] = json!(true);
    config["takeout_note"] = json!("Bacs devant le portail, après 19 h.");
    guest_at(wednesday())
        .with_config(&config)
        .with_stay(StayContext {
            checkout_at: Some(at("2026-10-01T09:00:00Z")),
            ..StayContext::default()
        })
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("home card");
            let found = banner(&surface);
            assert_eq!(found["message"], "i18n:guest.collection.departureEve");
            let json = serde_json::to_string(&surface).expect("surface json");
            let highlight = json.find("Highlight").expect("la consigne mise en avant");
            // Mise en avant veut dire lue avant les bacs, pas rangée à la fin de la carte.
            assert!(
                highlight < json.find("ColorDotItem").expect("des bacs"),
                "{json}"
            );
        });
}

/// The SDK renders the inactive state; the surface itself is not called.
#[test]
#[serial]
fn an_inactive_module_shows_the_sdk_state() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_module_status(ModuleStatus {
            active: false,
            workspace_enabled: true,
            incomplete: false,
            requires_config: true,
            missing_required_keys: Vec::new(),
        })
        .run(|ctx| {
            let surface = portaki_sdk::guest_shell::render(ctx, "home.card", render_home_card);
            assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
        });
}

/// A host writing in English: the French title, items and schedule stay, and so do the id and
/// the color; rows keep their place.
#[test]
#[serial]
fn a_save_in_english_keeps_the_french() {
    // L'inventaire des textes traduits : les champs du composteur et des points d'apport en font
    // partie, donc un hôte qui écrit en anglais garde aussi son français sur ceux-là.
    assert_eq!(
        config_save::localized_paths(EMISSIONS),
        [
            "bins.items",
            "bins.title",
            "collection_schedule",
            "compost_accepted",
            "compost_location",
            "compost_refused",
            "dropoff_points.note",
            "dropoff_points.title",
            "takeout_note"
        ]
    );
    let stored = json!({
        "bins": [
            { "id": "yellow", "title": { "fr": "Bac jaune", "en": "Yellow bin" },
              "items": { "fr": "Plastique\nCarton", "en": "Plastic\nCardboard" }, "color": "yellow" },
            { "title": "", "items": "" },
            { "id": "glass", "title": { "fr": "Verre" }, "items": { "fr": "Bouteilles" }, "color": "green" }
        ],
        "collection_schedule": { "fr": "Mardi", "en": "Tuesday" }
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|mut ctx| {
            ctx.locale = "en-US".into();
            let surface = render_host_main(ctx).expect("host main");
            let sent = config_save::form_args(&surface);
            // Stored order, the blank row where it was; ids on the filled rows only.
            assert_eq!(sent["bins"][0]["id"], "yellow");
            assert_eq!(sent["bins"][0]["items"], "Plastic\nCardboard");
            assert!(sent["bins"][1].get("id").is_none());
            assert_eq!(sent["bins"][2]["id"], "glass");
            assert_eq!(sent["bins"].as_array().unwrap().len(), 6);

            let saved = config_save::save(EMISSIONS, &surface, &stored, "en");
            assert_eq!(saved["bins"][0], stored["bins"][0]);
            assert_eq!(saved["bins"][2]["title"]["fr"], "Verre");
            assert_eq!(saved["bins"][2]["items"]["fr"], "Bouteilles");
            assert_eq!(saved["collection_schedule"], stored["collection_schedule"]);
        });
}

/// Le cas rural du §9 : pas de ramassage, deux points d'apport, un composteur.
///
/// Sans jour coché il n'y a plus de bandeau de collecte, et plus de ligne vide à la place.
#[test]
#[serial]
fn the_rural_case_shows_points_and_compost_without_a_collection_banner() {
    let config = json!({
        "dropoff_points": [
            { "title": "Parking du cimetière", "lat": 45.60, "lng": 6.00,
              "accepts_household": true, "accepts_packaging": true },
            { "title": "Salle des fêtes", "lat": 45.61, "lng": 6.01,
              "accepts_household": true, "accepts_glass": true }
        ],
        "compost_enabled": true,
        "compost_location": "Au fond du jardin, à gauche du portillon",
        "compost_accepted": "Épluchures\nMarc de café",
        "compost_refused": "Viande\nPlastique"
    });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .run(|ctx| {
            let card =
                serde_json::to_string(&render_home_card(ctx.clone()).expect("card")).unwrap();
            assert!(card.contains("Parking du cimetière"), "{card}");
            assert!(card.contains("guest.compost.title"), "{card}");
            // Aucun jour, aucune phrase : pas de bandeau, et surtout pas un bandeau vide.
            assert!(!card.contains("InfoBanner"), "{card}");

            let detail =
                serde_json::to_string(&render_explore_detail(ctx).expect("detail")).unwrap();
            assert!(detail.contains("guest.dropoff.title"), "{detail}");
            assert!(detail.contains("guest.compost.accepted"), "{detail}");
            assert!(detail.contains("Épluchures"), "{detail}");
        });
}

/// Ramassage **et** points d'apport : le bandeau revient, les points restent.
#[test]
#[serial]
fn collection_days_and_dropoff_points_live_together() {
    let config = json!({
        "collects_tue": true,
        "bins": [{ "title": "Bac jaune", "items": "Emballages", "color": "yellow" }],
        "dropoff_points": [
            { "title": "Parking du cimetière", "lat": 45.60, "lng": 6.00, "accepts_glass": true }
        ]
    });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .run(|ctx| {
            let card = serde_json::to_string(&render_home_card(ctx).expect("card")).unwrap();
            assert!(card.contains("Bac jaune"), "{card}");
            assert!(card.contains("Parking du cimetière"), "{card}");
        });
}

/// Le composteur ne s'affiche pas sans emplacement : le voyageur ne doit pas le chercher.
#[test]
#[serial]
fn a_compost_without_a_location_stays_hidden() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "compost_enabled": true,
            "bins": [{ "title": "Bac jaune", "items": "Emballages" }]
        }))
        .run(|ctx| {
            let card = serde_json::to_string(&render_home_card(ctx).expect("card")).unwrap();
            assert!(!card.contains("guest.compost.title"), "{card}");
        });
}

/// La porte de publication : une source suffit, et chacune des trois compte.
#[test]
#[serial]
fn one_source_is_enough_to_publish() {
    let ok_of = |config: Value| {
        let mut ok = false;
        MockContext::guest()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&config)
            .run(|ctx| {
                let readiness = publish_readiness(ctx).expect("readiness");
                ok = readiness
                    .items
                    .iter()
                    .find(|c| c.id == "where")
                    .expect("where")
                    .ok;
            });
        ok
    };

    assert!(!ok_of(json!({})), "rien du tout ne publie pas");
    assert!(
        ok_of(json!({ "collects_tue": true })),
        "des jours suffisent"
    );
    assert!(
        ok_of(json!({ "bins": [{ "title": "Bac jaune", "items": "Emballages" }] })),
        "un bac suffit"
    );
    assert!(
        ok_of(json!({
            "dropoff_points": [{ "title": "Parking", "lat": 45.6, "lng": 6.0, "accepts_glass": true }]
        })),
        "un point d'apport suffit"
    );
}

/// Un point nommé dont on ne dit pas ce qu'il accepte est signalé — recommandé, pas bloquant.
#[test]
#[serial]
fn a_point_that_takes_nothing_is_reported() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "dropoff_points": [{ "title": "Parking du cimetière", "lat": 45.6, "lng": 6.0 }]
        }))
        .run(|ctx| {
            let readiness = publish_readiness(ctx).expect("readiness");
            let accepts = readiness
                .items
                .iter()
                .find(|check| check.id == "dropoffAccepts")
                .expect("le défaut est signalé");
            assert!(!accepts.ok);
            // Le module reste publiable : le point existe, il manque juste son contenu.
            assert!(readiness.items.iter().find(|c| c.id == "where").unwrap().ok);
        });
}

/// Les points situés partent sur la carte du livret ; un point sans position n'y figure pas.
#[test]
#[serial]
fn only_located_points_reach_the_booklet_map() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({
            "dropoff_points": [
                { "title": "Parking du cimetière", "lat": 45.60, "lng": 6.00, "accepts_glass": true },
                { "title": "Sans position", "accepts_glass": true }
            ]
        }))
        .run(|ctx| {
            let response = waste_recycling::map_markers(ctx).expect("markers");
            assert_eq!(response.markers.len(), 1);
            assert_eq!(
                response.markers[0].label.as_deref(),
                Some("Parking du cimetière")
            );
        });
}
