//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use facility_hours::{render_explore_detail, render_home_card, render_host_main};
use portaki_sdk::context::StayContext;
use portaki_sdk::host::module::ModuleStatus;
use portaki_sdk::prelude::{DateTime, Utc, Uuid};
use portaki_test_utils::{MockContext, Property, SurfaceAssertions};
use serde_json::{json, Value};

#[path = "../../../support/config_form.rs"]
mod config_form;
#[path = "../../../support/config_save.rs"]
mod config_save;

const EMISSIONS: &str = concat!(env!("OUT_DIR"), "/portaki-emissions");

/// Un instant RFC 3339, en UTC.
fn at(rfc3339: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(rfc3339)
        .expect("date")
        .with_timezone(&Utc)
}

fn sample_config() -> Value {
    json!({
        "facilities": [
            { "id": "pool", "title": { "fr": "Piscine", "en": "Pool" }, "hours": "08:00 – 20:00", "lines": { "fr": "Maillot obligatoire", "en": "Swimwear required" } },
            { "title": "Accueil", "hours": "à partir de 16:00" }
        ],
        "general_note": "Horaires indicatifs"
    })
}

#[test]
#[serial]
fn home_card_empty_without_config() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            assert!(
                SurfaceAssertions::new(&render_home_card(ctx).expect("home card"))
                    .contains_type("EmptyState")
            );
        });
}

#[test]
#[serial]
fn home_card_uses_key_value_and_page_overlay() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("home card");
            assert!(SurfaceAssertions::new(&surface).contains_type("KeyValue"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("explore.detail"));
            assert!(json.contains("bottomSheet"));
        });
}

/// Cinq lignes : la carte en montre trois et propose le reste (§2.6).
#[test]
#[serial]
fn the_home_card_stops_at_three_rows_and_offers_the_rest() {
    let five = json!({
        "facilities": [
            { "title": "Piscine", "hours": "08:00 – 20:00" },
            { "title": "Salle de sport", "hours": "06:00 – 22:00" },
            { "title": "Accueil", "hours": "à partir de 16:00" },
            { "title": "Spa", "hours": "10:00 – 19:00" },
            { "title": "Parking", "hours": "24 h/24" }
        ]
    });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&five)
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("home card");
            let json = serde_json::to_string(&surface).expect("json");

            assert!(json.contains("Piscine"));
            assert!(json.contains("Accueil"));
            // La quatrième et la cinquième attendent dans la feuille.
            assert!(!json.contains("Spa"), "{json}");
            assert!(!json.contains("Parking"), "{json}");
            assert!(SurfaceAssertions::new(&surface).contains_type("Button"));
        });
}

/// Trois lignes ou moins : pas de bouton. Il promettrait une liste identique à celle qu'on lit déjà.
#[test]
#[serial]
fn no_button_when_the_card_already_shows_everything() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("home card");

            assert!(!SurfaceAssertions::new(&surface).contains_type("Button"));
        });
}

/// La feuille montre tout, bouton compris — c'est elle, « tous les horaires ».
#[test]
#[serial]
fn the_sheet_shows_every_row() {
    let five = json!({
        "facilities": [
            { "title": "Piscine", "hours": "08:00 – 20:00" },
            { "title": "Salle de sport", "hours": "06:00 – 22:00" },
            { "title": "Accueil", "hours": "à partir de 16:00" },
            { "title": "Spa", "hours": "10:00 – 19:00" }
        ]
    });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&five)
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("detail");
            let json = serde_json::to_string(&surface).expect("json");

            assert!(json.contains("Spa"), "{json}");
            assert!(
                !SurfaceAssertions::new(&surface).contains_type("Button"),
                "{json}"
            );
        });
}

/// Une ligne avec des heures structurées affiche son état et sa semaine (§2.6).
#[test]
#[serial]
fn structured_hours_carry_a_state_and_the_week() {
    let structured = json!({
        "facilities": [
            { "title": "Piscine", "opens_at": "08:00", "closes_at": "20:00" }
        ]
    });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&structured)
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("detail");
            let json = serde_json::to_string(&surface).expect("json");

            // Le badge dit l'un des trois états du §2.6, selon l'heure qu'il est vraiment.
            assert!(
                json.contains("guest.state.open")
                    || json.contains("guest.state.closed")
                    || json.contains("Ouvre à")
                    || json.contains("guest.state.opensAt"),
                "{json}"
            );
            // Les sept jours, avec le jour courant marqué.
            assert!(json.contains("\"details\""), "{json}");
            assert!(json.contains("\"current\":true"), "{json}");
        });
}

/// La prose d'un hôte n'est pas touchée : pas d'état deviné, pas de semaine inventée.
#[test]
#[serial]
fn a_prose_row_keeps_its_sentence_and_gains_no_state() {
    let prose = json!({
        "facilities": [
            { "title": "Accueil", "hours": "à partir de 16:00" }
        ]
    });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&prose)
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("card");
            let json = serde_json::to_string(&surface).expect("json");

            assert!(json.contains("à partir de 16:00"), "{json}");
            assert!(!json.contains("guest.state."), "{json}");
            assert!(!json.contains("\"details\""), "{json}");
        });
}

/// Ouvert en continu : toujours « Ouvert », sans heures à comparer (§6).
#[test]
#[serial]
fn around_the_clock_always_reads_open() {
    let always = json!({
        "facilities": [
            { "title": "Parking", "all_day": true }
        ]
    });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&always)
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("card");
            let json = serde_json::to_string(&surface).expect("json");

            assert!(json.contains("guest.state.open"), "{json}");
            assert!(!json.contains("guest.state.closed"), "{json}");
        });
}

#[test]
#[serial]
fn detail_enriched_list() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("detail");
            assert!(SurfaceAssertions::new(&surface).contains_type("ListItem"));
            assert!(SurfaceAssertions::new(&surface).contains_type("InfoBanner"));
        });
}

#[test]
#[serial]
fn the_host_form_sends_the_declared_keys() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            config_form::assert_form_matches_config(
                concat!(env!("OUT_DIR"), "/portaki-emissions"),
                &surface,
                &[],
            );
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("Piscine"));
            assert!(json.contains("Accueil"));
            assert!(json.contains("Horaires indicatifs"));
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
fn the_form_draws_the_facilities_the_host_has() {
    MockContext::host().with_config(&json!({})).run(|ctx| {
        let json = serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
        assert!(json.contains("facilities.0.title"), "{json}");
        assert!(!json.contains("facilities.1.title"), "{json}");
        assert!(json.contains(r#""facilities_count":2"#), "{json}");
    });

    let rows: Vec<serde_json::Value> = (0..facility_hours::MAX_FACILITIES)
        .map(|i| json!({ "title": format!("Équipement {i}") }))
        .collect();
    MockContext::host()
        .with_config(&json!({ "facilities": rows }))
        .run(|ctx| {
            let json =
                serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
            assert!(json.contains("facilities.11.title"), "{json}");
            assert!(!json.contains("facilities.12.title"), "{json}");
            assert!(
                json.contains(&format!(
                    r#""facilities_count":{}"#,
                    facility_hours::MAX_FACILITIES
                )),
                "{json}"
            );
        });
}

/// A host writing in English: the French title, lines and note stay, and so does the id; rows
/// keep their place.
#[test]
#[serial]
fn a_save_in_english_keeps_the_french() {
    assert_eq!(
        config_save::localized_paths(EMISSIONS),
        [
            "facilities.lines",
            "facilities.note",
            "facilities.title",
            "general_note"
        ]
    );
    let stored = json!({
        "facilities": [
            { "id": "pool", "title": { "fr": "Piscine", "en": "Pool" }, "hours": "9 h – 20 h",
              "lines": { "fr": "Tous les jours\nEnfants accompagnés", "en": "Every day\nChildren with an adult" },
              "note": { "fr": "Bonnet", "en": "Cap" } },
            { "title": "", "hours": "" },
            { "id": "spa", "title": { "fr": "Spa" }, "hours": "10 h – 19 h" }
        ],
        "general_note": { "fr": "Horaires indicatifs", "en": "Indicative hours" }
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|mut ctx| {
            ctx.locale = "en-US".into();
            let surface = render_host_main(ctx).expect("host main");
            let sent = config_save::form_args(&surface);
            // Stored order, the blank row where it was; ids on the filled rows only.
            assert_eq!(sent["facilities"][0]["id"], "pool");
            assert_eq!(sent["facilities"][0]["title"], "Pool");
            assert_eq!(
                sent["facilities"][0]["lines"],
                "Every day\nChildren with an adult"
            );
            assert_eq!(sent["facilities"][0]["note"], "Cap");
            assert!(sent["facilities"][1].get("id").is_none());
            assert_eq!(sent["facilities"][2]["id"], "spa");
            // Trois lignes stockées, trois dessinées : plus d'emplacements vides en bout de liste.
            assert_eq!(sent["facilities"].as_array().unwrap().len(), 3);
            assert_eq!(sent["general_note"], "Indicative hours");

            let saved = config_save::save(EMISSIONS, &surface, &stored, "en");
            // Ce que ce test protège : une sauvegarde en anglais ne perd rien de ce qui était
            // écrit. Chaque clé déjà stockée est comparée une à une, plutôt que l'objet entier —
            // le formulaire renvoie aussi les champs d'horaires structurés, vides ici, et une
            // égalité stricte se serait cassée pour tout module ajoutant un champ de ligne.
            for (key, value) in stored["facilities"][0].as_object().expect("stored row") {
                assert_eq!(&saved["facilities"][0][key], value, "clé {key}");
            }
            assert_eq!(saved["facilities"][2]["title"]["fr"], "Spa");
            assert_eq!(saved["general_note"], stored["general_note"]);
        });
}

const FR_BUNDLE: &str = include_str!("../i18n/fr-FR.json");
const EN_BUNDLE: &str = include_str!("../i18n/en-US.json");

/// Un mot rendu qui ressemble à une clé : un point au milieu, et des lettres autour.
///
/// Le test frère de `local-guide` cherchait la clé dans le libellé entier ; ici elle est collée à
/// l'heure — « guest.stay.from 17:00 » — et l'espace suffisait à la faire passer. On regarde donc
/// chaque mot. « 17:00 » n'a pas de point, « 9 h – 20 h » non plus.
fn looks_like_a_key(word: &str) -> bool {
    // Le point final d'une phrase n'en fait pas une clé : « Bonnet de bain non obligatoire. »
    word.trim_end_matches('.').contains('.') && word.contains(char::is_alphabetic)
}

/// Les `key` et `value` des `KeyValue` de l'arbre, où qu'ils soient.
fn key_value_texts(value: &Value, into: &mut Vec<String>) {
    match value {
        Value::Object(fields) => {
            if fields.get("type").and_then(Value::as_str) == Some("KeyValue") {
                for field in ["key", "value"] {
                    if let Some(text) = fields.get(field).and_then(Value::as_str) {
                        into.push(text.to_string());
                    }
                }
            }
            fields
                .values()
                .for_each(|field| key_value_texts(field, into));
        }
        Value::Array(items) => items.iter().for_each(|item| key_value_texts(item, into)),
        _ => {}
    }
}

/// Les tuiles du séjour portent un texte, jamais la clé qui le désigne.
///
/// La batterie de conformité rend chaque surface sur un mock vide : sans séjour, la grille
/// disparaît et ses clés ne sont jamais demandées. Et `t!` ne rend pas d'`Err` sur une clé
/// absente — l'hôte comme le mock répondent la clé elle-même, si bien que le repli de
/// `stay_tile` ne se déclenchait pas et que le voyageur lisait « guest.stay.from 17:00 ».
/// Les traductions sont chargées comme dans les aperçus : sans elles, une clé présente et une
/// clé absente se rendent pareil.
#[test]
#[serial]
fn the_stay_tiles_never_show_an_i18n_key() {
    let fr: Value = serde_json::from_str(FR_BUNDLE).expect("bundle");
    let stay = StayContext {
        stay_id: Uuid::from_u128(0x3333_3333_3333_3333_3333_3333_3333_3333),
        checkin_at: Some(at("2026-06-01T15:00:00Z")),
        checkout_at: Some(at("2026-06-08T10:00:00Z")),
        ..StayContext::default()
    };
    let context = fr
        .as_object()
        .expect("bundle object")
        .iter()
        .fold(MockContext::guest(), |builder, (key, text)| {
            builder.with_translation(key, text.as_str().unwrap_or_default())
        })
        .with_capabilities(&[capability::core::STORAGE])
        .with_property(Property::default())
        .with_stay(stay)
        .with_config(&sample_config());

    let (surface, asked) = context.run_with(|ctx, host| {
        let surface = render_home_card(ctx).expect("home card");
        (surface, host.translated_keys())
    });

    let mut texts = Vec::new();
    key_value_texts(
        &serde_json::to_value(&surface.root).expect("arbre SDUI"),
        &mut texts,
    );
    // Les deux tuiles du séjour, et leur heure : sans elles le test ne garde rien.
    assert!(
        texts.iter().any(|text| text.contains("17:00")),
        "la carte n'a pas la tuile d'arrivée — {texts:?}"
    );
    assert!(
        texts.iter().any(|text| text.contains("12:00")),
        "la carte n'a pas la tuile de départ — {texts:?}"
    );

    for text in &texts {
        match text.strip_prefix("i18n:") {
            // Une clé laissée à la coquille : elle doit exister partout, sinon elle fuit aussi.
            Some(key) => assert!(
                fr.get(key).is_some(),
                "la tuile renvoie à une clé absente — {key}"
            ),
            None => {
                for word in text.split_whitespace() {
                    assert!(!looks_like_a_key(word), "la tuile porte une clé — {text}");
                }
            }
        }
    }

    // Et toute clé que la carte a fait traduire existe en fr comme en en.
    assert!(!asked.is_empty(), "la carte n'a fait traduire aucune clé");
    for raw in [FR_BUNDLE, EN_BUNDLE] {
        let bundle: Value = serde_json::from_str(raw).expect("bundle");
        for key in &asked {
            assert!(bundle.get(key).is_some(), "clé manquante — {key}");
        }
    }
}
