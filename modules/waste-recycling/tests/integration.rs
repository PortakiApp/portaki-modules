//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use portaki_sdk::contracts::publish::PublishLevel;
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
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
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

/// Le code et les heures du local passent **avant** le chemin (§2.7).
///
/// Avant, parce qu'un local verrouillé ou fermé arrête le trajet avant qu'il commence : descendre
/// trois étages pour lire « fermé le dimanche » en bas est ce qu'un livret existe pour éviter.
/// Et rien des trois renseigné : pas de carte, qui promettrait un local.
#[test]
#[serial]
fn the_bin_room_says_its_code_and_its_hours_before_the_way_there() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_stay(StayContext {
            checkin_at: Some(at("2026-10-10T14:00:00Z")),
            checkout_at: Some(at("2026-10-12T08:00:00Z")),
            ..StayContext::default()
        })
        .with_now(at("2026-10-11T10:00:00Z"))
        .with_config(&json!({
            "bin_room_steps": { "fr": "Derrière la haie\nPorte grise" },
            "bin_room_code": "1234A",
            "bin_room_hours": { "fr": "7 h – 21 h, fermé le dimanche" }
        }))
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("detail");
            let json_text = serde_json::to_string(&surface).unwrap();
            let code = json_text.find("1234A").expect("le code");
            let hours = json_text.find("fermé le dimanche").expect("les heures");
            let way = json_text.find("Derrière la haie").expect("le chemin");
            assert!(code < hours && hours < way, "{json_text}");
            // Pendant le séjour : révélé, et copiable — on le lit devant un digicode.
            assert!(json_text.contains("\"copy\":true"), "{json_text}");
            assert!(!json_text.contains("\"secret\""), "{json_text}");
        });

    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "collection_schedule": { "fr": "Mardi et vendredi" } }))
        .run(|ctx| {
            let json_text =
                serde_json::to_string(&render_explore_detail(ctx).expect("detail")).unwrap();
            assert!(
                !json_text.contains("i18n:guest.binRoom.title"),
                "{json_text}"
            );
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
            // Les bacs dans leur carte, comme les deux blocs qui les précèdent (§2.7).
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("guest.bins.title"), "{json}");
            assert!(json.contains(r#""swatch":"yellow""#), "{json}");
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
            "collection_schedule": "Mardi",
            // Ouverts : leurs champs ne sont dessinés que là.
            "bin_room_enabled": true,
            "compost_enabled": true
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
                highlight < json.find(r#""swatch""#).expect("des bacs"),
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

/// Les lignes suivent ce que l'hôte a saisi, et « Ajouter » en demande une de plus.
#[test]
#[serial]
fn the_form_draws_the_bins_the_host_has() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .run(|ctx| {
            let json =
                serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
            assert!(json.contains("bins.0.title"), "{json}");
            assert!(!json.contains("bins.1.title"), "{json}");
            assert!(json.contains(r#""bins_count":2"#), "{json}");
        });

    let rows: Vec<Value> = (0..waste_recycling::MAX_BINS)
        .map(|i| json!({ "title": format!("Bac {i}"), "items": "Tout" }))
        .collect();
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "bins": rows }))
        .run(|ctx| {
            let json =
                serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
            assert!(json.contains("bins.11.title"), "{json}");
            assert!(!json.contains("bins.12.title"), "{json}");
            assert!(
                json.contains(&format!(r#""bins_count":{}"#, waste_recycling::MAX_BINS)),
                "{json}"
            );
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
            "bin_room_hours",
            "bin_room_steps",
            "bin_room_where",
            "bins.items",
            "bins.location",
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
            // Trois lignes stockées, trois dessinées : plus d'emplacements vides en bout de liste.
            assert_eq!(sent["bins"].as_array().unwrap().len(), 3);

            let saved = config_save::save(EMISSIONS, &surface, &stored, "en");
            // Le texte stocké est intact, champ par champ. Pas d'égalité d'objets : « où il se
            // trouve » est un champ localisé neuf, et un enregistrement écrit sa clé vide dans
            // chaque ligne — comme pour tout champ localisé ajouté à une ligne.
            assert_eq!(saved["bins"][0]["id"], stored["bins"][0]["id"]);
            assert_eq!(saved["bins"][0]["title"], stored["bins"][0]["title"]);
            assert_eq!(saved["bins"][0]["items"], stored["bins"][0]["items"]);
            assert_eq!(saved["bins"][0]["color"], stored["bins"][0]["color"]);
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
            // Et pas de bouton « où déposer » : sans bacs, les rangées *sont* la carte, et le
            // bouton n'enverrait voir que ce qu'elle montre déjà.
            assert!(!card.contains("guest.dropoff.cta"), "{card}");

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
            let card =
                serde_json::to_string(&render_home_card(ctx.clone()).expect("card")).unwrap();
            assert!(card.contains("Bac jaune"), "{card}");
            // Les points ne sont plus recopiés sous les bacs : un bouton y mène, et la feuille
            // les montre avec leur plan et leurs distances.
            assert!(!card.contains("Parking du cimetière"), "{card}");
            assert!(card.contains("guest.dropoff.cta"), "{card}");

            let detail =
                serde_json::to_string(&render_explore_detail(ctx).expect("detail")).unwrap();
            assert!(detail.contains("Parking du cimetière"), "{detail}");
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

/// Un point nommé dont on ne dit pas ce qu'il accepte bloque, sous sa première case (§2.4).
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
                .find(|check| check.id == "config.dropoff_points.0.accepts_household")
                .expect("le défaut est signalé");
            assert!(!accepts.ok);
            assert_eq!(accepts.level, PublishLevel::Required);
            assert_eq!(
                accepts.hint.get("fr"),
                "Choisissez au moins un type de déchets."
            );
            // Il y a bien une source : seul le point est en défaut.
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

/// Trois bacs sur la carte, le reste derrière « Où déposer mes déchets ? » (§1.9, §2.7).
///
/// Un logement qui trie le verre, le papier, les emballages, les biodéchets et le tout-venant en
/// a cinq : les empiler poussait le bouton — donc le local et le plan — sous le pli.
#[test]
#[serial]
fn the_card_shows_three_bins_and_a_way_to_the_rest() {
    let five = json!({
        "bins": (0..5)
            .map(|index| json!({
                "id": format!("bin{index}"),
                "title": {"fr": format!("Bac {index}"), "en": format!("Bin {index}")},
                "items": {"fr": "Contenu", "en": "Contents"},
                "color": "#888888"
            }))
            .collect::<Vec<_>>(),
        "collection_schedule": "Mardi matin"
    });

    MockContext::guest().with_config(&five).run(|ctx| {
        let card = serde_json::to_string(&render_home_card(ctx).expect("carte")).expect("json");
        assert_eq!(card.matches("\"type\":\"ListItem\"").count(), 3, "{card}");
        assert!(card.contains("guest.dropoff.cta"), "{card}");
    });
}

/// Deux bacs, aucun local, aucun point d'apport : rien derrière, donc pas de bouton.
///
/// Un bouton vers une section vide promet un local que l'hôte n'a pas renseigné.
#[test]
#[serial]
fn no_button_when_the_sheet_has_nothing_more_to_show() {
    MockContext::guest()
        .with_config(&sample_config())
        .run(|ctx| {
            let card = serde_json::to_string(&render_home_card(ctx).expect("carte")).expect("json");
            assert_eq!(card.matches("\"type\":\"ListItem\"").count(), 2, "{card}");
            assert!(!card.contains("guest.dropoff.cta"), "{card}");
        });
}

/// Le code du local à `now`, pour un séjour du 10 octobre 16 h au 12 octobre 10 h (Paris).
fn bin_room_code_at(now: &str, stay: bool) -> String {
    let mut mock = MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_now(at(now))
        // En clair et sans préfixe `enc:v1:` : ce que la plateforme rend d'un code enregistré
        // avant qu'il ne soit déclaré secret.
        .with_config(&json!({
            "bin_room_where": { "fr": "Au fond de la cour" },
            "bin_room_code": "1234A"
        }));
    if stay {
        mock = mock.with_stay(StayContext {
            checkin_at: Some(at("2026-10-10T14:00:00Z")),
            checkout_at: Some(at("2026-10-12T08:00:00Z")),
            ..StayContext::default()
        });
    }
    mock.run(|ctx| serde_json::to_string(&render_explore_detail(ctx).expect("detail")).unwrap())
}

/// Spec Tri §2.3 : le code du local suit la révélation d'Accès — masqué, avec sa date, jusqu'à la
/// veille 16 h ; en clair et copiable ensuite ; masqué de nouveau après le départ.
#[test]
#[serial]
fn the_bin_room_code_waits_for_the_reveal() {
    let before = bin_room_code_at("2026-10-09T13:59:00Z", true);
    assert!(!before.contains("1234A"), "{before}");
    assert!(before.contains("\"revealed\":false"), "{before}");
    assert!(
        before.contains("\"reveal_at\":\"9 oct. 2026 · 16:00\""),
        "{before}"
    );
    assert!(!before.contains("\"copy\":true"), "{before}");

    let after = bin_room_code_at("2026-10-09T14:00:00Z", true);
    assert!(after.contains("1234A"), "{after}");
    assert!(after.contains("\"copy\":true"), "{after}");
    assert!(!after.contains("\"secret\""), "{after}");

    let gone = bin_room_code_at("2026-10-12T08:00:01Z", true);
    assert!(!gone.contains("1234A"), "{gone}");
    assert!(gone.contains("\"revealed\":false"), "{gone}");
    assert!(!gone.contains("reveal_at"), "{gone}");

    // Sans séjour (lien d'aperçu, livret sans arrivée) : masqué, sans date promise.
    let nowhere = bin_room_code_at("2026-10-11T10:00:00Z", false);
    assert!(!nowhere.contains("1234A"), "{nowhere}");
    assert!(!nowhere.contains("reveal_at"), "{nowhere}");
}

/// Un code enregistré en clair avant ce marquage : déclaré `secret` (la plateforme le chiffre à
/// la prochaine écriture), et un enregistrement du formulaire — qui ne renvoie jamais le code —
/// le garde.
#[test]
#[serial]
fn a_plaintext_code_saved_before_survives_the_secret_form() {
    let field = config_save::declared_fields(EMISSIONS)
        .into_iter()
        .find(|f| f["key"] == "bin_room_code")
        .expect("bin_room_code déclaré");
    assert_eq!(field["type"], "secret", "{field}");
    assert_eq!(
        field["reveal"],
        json!(["guest_pre_arrival", "guest_stay"]),
        "{field}"
    );

    let stored = json!({
        "bin_room_enabled": true,
        "bin_room_where": { "fr": "Au fond de la cour" },
        "bin_room_code": "1234A"
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            assert!(!serde_json::to_string(&surface).unwrap().contains("1234A"));
            let saved = config_save::save(EMISSIONS, &surface, &stored, "fr");
            assert_eq!(saved["bin_room_code"], "1234A");
        });
}
