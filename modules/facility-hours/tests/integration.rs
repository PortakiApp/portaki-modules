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
#[allow(dead_code)]
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
        // Une heure fixe, en pleine journée : l'heure réelle du run faisait tomber le test dans la
        // dernière heure d'ouverture (« Ferme dans N min ») le soir.
        .with_now(at("2026-03-11T10:00:00Z"))
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("detail");
            let json = serde_json::to_string(&surface).expect("json");

            assert!(json.contains("guest.state.open"), "{json}");
            // Les sept jours, avec le jour courant marqué — et nommé « aujourd'hui », pas
            // « mercredi » : un rendu figé qui nomme le jour ment six jours sur sept.
            assert!(json.contains("\"details\""), "{json}");
            assert!(
                json.contains(
                    "\"label\":\"i18n:guest.today\",\"value\":\"08:00 – 20:00\",\"current\":true"
                ),
                "{json}"
            );
            assert!(
                !json.contains("mercredi") && !json.contains("Wednesday"),
                "{json}"
            );
        });
}

/// Une heure avant la fermeture, le badge dit combien de temps il reste (§9 #7) ; hors saison,
/// il dit quand la ligne rouvre (§9 #4), et non plus seulement « Hors saison ».
#[test]
#[serial]
fn the_badge_says_closing_soon_and_when_the_season_reopens() {
    let config = json!({
        "facilities": [
            { "title": "Piscine", "opens_at": "08:00", "closes_at": "20:00" },
            { "title": "Sauna", "opens_at": "08:00", "closes_at": "20:00",
              "season_from": "06-01", "season_to": "09-30" }
        ]
    });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        // 19:20 à Paris : la piscine ferme dans 40 min, le sauna attend juin.
        .with_now(at("2026-03-11T18:20:00Z"))
        .run(|ctx| {
            let json = serde_json::to_string(&render_explore_detail(ctx).expect("detail")).unwrap();
            // Le mock rend la clé d'un `t!` paramétré : c'est elle qu'on lit.
            assert!(json.contains("\"guest.state.closesSoon\""), "{json}");
            assert!(json.contains("\"tone\":\"warning\""), "{json}");
            assert!(json.contains("\"guest.state.closedUntil\""), "{json}");
            assert!(!json.contains("guest.state.outOfSeason"), "{json}");
        });
}

/// Les deux textes existent dans les dix langues, avec leur variable : une clé absente passerait
/// tous les autres contrôles, le mock rendant la clé elle-même.
#[test]
fn the_new_badges_are_in_every_bundle() {
    for bundle in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/i18n")).unwrap() {
        let path = bundle.unwrap().path();
        let texts: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        for (key, variable) in [
            ("guest.state.closesSoon", "{minutes}"),
            ("guest.state.closedUntil", "{date}"),
            ("host.facility.ranges.overlap", ""),
            ("guest.state.openUntil", "{time}"),
            ("host.facility.ranges.overlapOn", "{day}"),
            ("host.exception.past", ""),
            ("guest.today", ""),
        ] {
            let text = texts[key].as_str().unwrap_or_default();
            assert!(
                !text.is_empty() && text.contains(variable),
                "{path:?} {key}"
            );
        }
    }
}

/// La carte d'accueil dit aussi jusqu'à quand, et déplie la semaine (§2.6).
///
/// L'état remplaçait l'horaire sur la carte. Mais « Ouvert » ne dit pas jusqu'à quand, et c'est
/// la question suivante : il fallait ouvrir la feuille pour la poser.
#[test]
#[serial]
fn the_card_says_until_when_and_unfolds_the_week() {
    let structured = json!({
        "facilities": [
            { "title": "Piscine", "opens_at": "08:00", "closes_at": "20:00" }
        ]
    });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&structured)
        .run(|ctx| {
            let json = serde_json::to_string(&render_home_card(ctx).expect("card")).unwrap();
            assert!(json.contains("08:00"), "{json}");
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

/// Les cas du §9 côté livret : 24 h/24 sans statut, une plage qui passe minuit ouverte à 1 h et
/// « ferme à 02:00 », trois plages dans le détail, et la fermeture exceptionnelle du jour en
/// avertissement avec son motif. Une fermeture passée ne change plus rien.
#[test]
#[serial]
fn the_special_cases_reach_the_booklet() {
    let fr: Value = serde_json::from_str(FR_BUNDLE).expect("bundle");
    fr.as_object()
        .expect("bundle object")
        .iter()
        .fold(MockContext::guest(), |builder, (key, text)| {
            builder.with_translation(key, text.as_str().unwrap_or_default())
        })
        .with_capabilities(&[capability::core::STORAGE])
        .with_property(Property::default())
        // Mercredi 15 juillet 2026, 1 h à Paris.
        .with_now(at("2026-07-14T23:00:00Z"))
        .with_config(&json!({ "facilities": [
            { "title": "Parking", "mode": "always" },
            { "title": "Bar", "range_1": "22:00-02:00" },
            { "title": "Boutique", "range_1": "08:00-10:00", "range_2": "12:00-14:00",
              "range_3": "18:00-22:00" },
            { "title": "Piscine", "range_1": "09:00-19:00", "exceptions": [
                { "date": "2026-07-15", "closed": true, "reason": "Entretien du bassin" },
                { "date": "2026-07-01", "closed": true, "reason": "Ancienne fermeture" }
            ] }
        ] }))
        .run(|mut ctx| {
            ctx.locale = "fr-FR".into();
            let json =
                serde_json::to_string(&render_explore_detail(ctx).expect("detail")).expect("json");
            assert!(json.contains("i18n:guest.allDay"), "{json}");
            assert!(json.contains("Ferme dans 60 min"), "{json}");
            assert!(
                json.contains("08:00 – 10:00 · 12:00 – 14:00 · 18:00 – 22:00"),
                "{json}"
            );
            assert!(json.contains("Entretien du bassin"), "{json}");
            assert!(!json.contains("Ancienne fermeture"), "{json}");
            assert!(json.contains(r#""tone":"warning""#), "{json}");
            // 24 h/24 : pas de badge « Ouvert ».
            assert!(!json.contains("Ouvert"), "{json}");
        });
}

/// La page : deux sections, le tableau des équipements et le panneau de chaque ligne nommé
/// `facilities.<i>.<clé>`. « Ajouter » ouvre une ligne de plus, sa plage par défaut remplie ; à
/// trente lignes, plus de bouton.
#[test]
#[serial]
fn the_settings_page_is_two_sections_and_a_table() {
    MockContext::host()
        .with_config(&json!({ "facilities": [
            { "id": "pool", "title": "Piscine", "group": "equipment", "range_1": "09:00-19:00" },
            { "title": "", "hours": "" },
            { "id": "desk", "title": "Réception", "mode": "on_request", "note": "Appelez le 12" }
        ] }))
        .run(|ctx| {
            let json =
                serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
            assert_eq!(json.matches(r#""type":"Section""#).count(), 2, "{json}");
            for needle in [
                r#""rowName":"facilities""#,
                r#""filterKey":"group""#,
                r#""id":"pool""#,
                r#""name":"facilities.0.range_1""#,
                r#""visibleWhen":"facilities.0.mode=same""#,
                r#""name":"facilities.1.week""#,
                r#""visibleWhen":"facilities.1.mode=on_request""#,
                r#""name":"facilities.1.exceptions""#,
                r#""name":"card_limit""#,
                "09:00 – 19:00",
                "\"2 / 30\"",
                "newFacilities",
            ] {
                assert!(json.contains(needle), "{needle} absent : {json}");
            }
            // L'emplacement vide de l'ancien formulaire n'est pas une ligne.
            assert!(!json.contains("facilities.2.title"), "{json}");
        });

    MockContext::host().with_config(&json!({})).run(|mut ctx| {
        ctx.input = json!({ "newFacilities": "abc" });
        let json = serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
        assert!(json.contains(r#""id":"abc""#), "{json}");
        assert!(json.contains(r#""name":"facilities.0.title""#), "{json}");
        assert!(json.contains(r#""value":"09:00-19:00""#), "{json}");
    });

    let rows: Vec<Value> = (0..facility_hours::MAX_FACILITIES)
        .map(|i| json!({ "id": format!("f{i}"), "title": format!("Équipement {i}") }))
        .collect();
    MockContext::host()
        .with_config(&json!({ "facilities": rows }))
        .run(|ctx| {
            let json =
                serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
            assert!(json.contains("facilities.29.title"), "{json}");
            assert!(!json.contains("newFacilities"), "{json}");
            assert!(json.contains("host.facilities.tooMany"), "{json}");
        });
}

/// Une ligne d'avant les plages : son ouverture, sa fermeture et sa coupure dans les plages, sa
/// phrase dans un champ gardé avec un avertissement ; un mode sur demande montre sa note.
#[test]
#[serial]
fn an_old_row_shows_its_hours_as_ranges_and_keeps_its_sentence() {
    MockContext::host()
        .with_config(&json!({ "facilities": [
            { "title": "Accueil", "opens_at": "08:30", "closes_at": "18:00",
              "break_from": "12:00", "break_to": "14:00", "season_from": "06-01", "season_to": "09-30" },
            { "title": "Piscine", "hours": "9 h – 20 h" },
            { "title": "Spa", "mode": "by_day", "exceptions": [
                { "id": "mon", "opens_at": "09:00", "closes_at": "12:00" }
            ] }
        ] }))
        .run(|ctx| {
            let json = serde_json::to_string(&render_host_main(ctx).expect("host main"))
                .expect("json");
            for needle in [
                r#""value":"08:30-12:00""#,
                r#""value":"14:00-18:00""#,
                r#""value":"01/06""#,
                r#""value":"30/09""#,
                r#""name":"facilities.1.hours""#,
                r#""value":"mon=09:00-12:00""#,
                "08:30 – 12:00 · 14:00 – 18:00",
            ] {
                assert!(json.contains(needle), "{needle} absent : {json}");
            }
            // L'avertissement, pas une erreur : rien ne bloque la publication.
            assert!(json.contains("InlineNotice"), "{json}");
        });
}

/// « Aujourd'hui » au fuseau du logement : « Ouvert · ferme à 19:00 » à 10 h à Paris.
#[test]
#[serial]
fn the_today_column_reads_the_property_clock() {
    let fr: Value = serde_json::from_str(FR_BUNDLE).expect("bundle");
    fr.as_object()
        .expect("bundle object")
        .iter()
        .fold(MockContext::host(), |builder, (key, text)| {
            builder.with_translation(key, text.as_str().unwrap_or_default())
        })
        .with_property(Property::default())
        .with_now(at("2026-07-15T08:00:00Z"))
        .with_config(&json!({ "facilities": [
            { "id": "pool", "title": "Piscine", "range_1": "09:00-19:00" },
            { "id": "spa", "title": "Spa", "range_1": "09:00-10:40" },
            { "id": "sauna", "title": "Sauna", "season_from": "01/11", "season_to": "31/03",
              "range_1": "17:00-21:00" },
            { "id": "car", "title": "Parking", "mode": "always" },
            { "id": "desk", "title": "Ménage", "mode": "on_request", "note": "La veille" },
            { "id": "gym", "title": "Gym", "range_1": "09:00-19:00",
              "exceptions": [{ "date": "2026-07-15", "closed": true, "reason": "Travaux" }] }
        ] }))
        .run(|mut ctx| {
            ctx.locale = "fr-FR".into();
            let json =
                serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
            for needle in [
                "Ouvert · ferme à 19:00",
                "Ferme dans 40 min",
                "Fermé jusqu’au 1er nov.",
                "i18n:host.facility.mode.always",
                "i18n:host.facility.mode.on_request",
                "i18n:guest.state.closed",
            ] {
                assert!(json.contains(needle), "{needle} absent : {json}");
            }
        });
}

/// Une ligne jamais ouverte n'envoie que son id, que le tableau porte ; un hôte qui écrit en
/// anglais envoie son anglais, et la plateforme garde le français (fusion par langue).
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
            assert_eq!(sent["facilities"][0]["title"], "Pool");
            assert_eq!(
                sent["facilities"][0]["lines"],
                "Every day\nChildren with an adult"
            );
            assert_eq!(sent["facilities"][0]["note"], "Cap");
            // Deux lignes : l'emplacement vide est tombé, le Spa le suit.
            assert_eq!(sent["facilities"].as_array().unwrap().len(), 2);
            assert_eq!(sent["facilities"][1]["title"], "Spa");
            assert_eq!(sent["general_note"], "Indicative hours");
            // Les id viennent du tableau, ligne par ligne : la plateforme fusionne par id.
            let tree = serde_json::to_value(&surface).unwrap().to_string();
            assert!(tree.contains(r#""id":"pool""#), "{tree}");
            assert!(tree.contains(r#""id":"spa""#), "{tree}");
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
