//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use portaki_sdk::context::StayContext;
use portaki_sdk::host::module::ModuleStatus;
use portaki_sdk::prelude::{DateTime, Utc};
use portaki_test_utils::{MockContext, SurfaceAssertions};
use serde_json::{json, Value};

use waste_recycling::{render_explore_detail, render_home_card, render_host_main};

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
    assert_eq!(
        config_save::localized_paths(EMISSIONS),
        [
            "bins.items",
            "bins.title",
            "collection_schedule",
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
