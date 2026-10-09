//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use portaki_test_utils::{MockContext, SurfaceAssertions};
use safety_shutoffs::{
    publish_readiness, render_explore_detail, render_home_card, render_host_main,
};
use serde_json::{json, Value};
use serial_test::serial;

#[path = "../../../support/config_form.rs"]
mod config_form;
#[path = "../../../support/config_save.rs"]
mod config_save;

const EMISSIONS: &str = concat!(env!("OUT_DIR"), "/portaki-emissions");

fn sample_config() -> Value {
    json!({
        "shutoffs": [
            { "kind": "electricity", "title": "Tableau électrique", "location": "Placard de l’entrée",
              "instruction": "Disjoncteur principal en haut à gauche." },
            { "kind": "water", "title": "Vanne d’arrêt d’eau", "location": "Trappe des WC" }
        ],
        "general_note": "Coupez d’abord la vanne, puis appelez-moi."
    })
}

fn guest() -> portaki_test_utils::MockContextBuilder {
    MockContext::guest().with_capabilities(&[capability::core::STORAGE])
}

/// La carte d'accueil ne liste rien, et c'est le dessin : elle ouvre le plein écran (§2.22).
#[test]
#[serial]
fn the_home_card_says_nothing_and_opens_the_fullscreen_detail() {
    guest().with_config(&sample_config()).run(|ctx| {
        let surface = render_home_card(ctx).expect("home card");
        let json = serde_json::to_string(&surface).expect("json");
        assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
        // Aucun organe sur la carte : ni rangée, ni titre d'organe.
        assert!(
            !SurfaceAssertions::new(&surface).contains_type("ListItem"),
            "{json}"
        );
        assert!(!json.contains("Tableau électrique"), "{json}");
        assert!(json.contains("home.card.subtitle"), "{json}");
        assert!(json.contains(r#""presentation":"fullscreen""#), "{json}");
    });
}

/// Sans organe complet, les deux surfaces montrent l'état vide — pas une carte qui promet rien.
#[test]
#[serial]
fn nothing_complete_shows_the_empty_state() {
    for config in [json!({}), json!({ "shutoffs": [{ "title": "Vanne" }] })] {
        guest().with_config(&config).run(|ctx| {
            for surface in [
                render_home_card(ctx.clone()).expect("home card"),
                render_explore_detail(ctx.clone()).expect("detail"),
            ] {
                assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
            }
        });
    }
}

/// Le détail : le bandeau du 112, la consigne de l'hôte, puis un bloc par organe.
#[test]
#[serial]
fn the_detail_leads_with_the_112_banner_then_the_host_note() {
    guest().with_config(&sample_config()).run(|ctx| {
        let surface = render_explore_detail(ctx).expect("detail");
        let json = serde_json::to_string(&surface).expect("json");
        let danger = json.find("guest.danger.title").expect("bandeau danger");
        let note = json.find("guest.note.title").expect("bandeau consigne");
        assert!(
            danger < note,
            "la consigne de l'hôte passe après le 112 : {json}"
        );
        assert_eq!(json.matches(r#""tone":"danger""#).count(), 1, "{json}");
        assert!(json.contains(r#""tone":"info""#), "{json}");
        // L'emplacement en gras, la consigne en légende.
        assert!(json.contains(r#""text":"Placard de l’entrée""#), "{json}");
        assert!(json.contains(r#""emphasis":"strong""#), "{json}");
        // Le glyphe vient du type.
        assert!(json.contains(r#""icon":"zap""#), "{json}");
        assert!(json.contains(r#""icon":"droplet""#), "{json}");
    });
}

/// Pas de consigne générale : pas de bandeau d'information (cas du §9).
#[test]
#[serial]
fn without_a_general_note_there_is_no_info_banner() {
    let config = json!({
        "shutoffs": [{ "kind": "gas", "title": "Robinet de gaz", "location": "Sous l’évier" }]
    });
    guest().with_config(&config).run(|ctx| {
        let json =
            serde_json::to_string(&render_explore_detail(ctx).expect("detail")).expect("json");
        assert!(!json.contains("guest.note.title"), "{json}");
        assert!(json.contains("guest.danger.title"), "{json}");
    });
}

/// Un seul organe, sans consigne : un bloc, et rien d'autre (cas du §9).
#[test]
#[serial]
fn a_single_shutoff_without_an_instruction_renders_one_block() {
    let config = json!({
        "shutoffs": [{ "kind": "water", "title": "Vanne générale", "location": "Au sous-sol" }]
    });
    guest().with_config(&config).run(|ctx| {
        let surface = render_explore_detail(ctx).expect("detail");
        let json = serde_json::to_string(&surface).expect("json");
        assert_eq!(json.matches(r#""type":"Card""#).count(), 1, "{json}");
        assert_eq!(json.matches(r#""variant":"caption""#).count(), 1, "{json}");
    });
}

/// Un type inconnu, ou « autre », porte le glyphe d'information — jamais celui de l'électricité.
#[test]
#[serial]
fn an_other_kind_carries_the_info_glyph() {
    let config = json!({
        "shutoffs": [{ "kind": "boiler", "title": "Chaudière", "location": "Garage" }]
    });
    guest().with_config(&config).run(|ctx| {
        let json =
            serde_json::to_string(&render_explore_detail(ctx).expect("detail")).expect("json");
        assert!(json.contains(r#""icon":"info-circle""#), "{json}");
        assert!(!json.contains(r#""icon":"zap""#), "{json}");
    });
}

/// La barre du bas : sans téléphone d'hôte, un seul bouton, et il renvoie à la section Aide.
///
/// Le runtime ne sert pas encore `context.host` : c'est l'état d'aujourd'hui en production, et le
/// SDK le prescrit — pas de bouton d'appel plutôt qu'un bouton qui échoue.
#[test]
#[serial]
fn without_a_host_phone_only_the_emergency_button_shows() {
    guest().with_config(&sample_config()).run(|ctx| {
        let json =
            serde_json::to_string(&render_explore_detail(ctx).expect("detail")).expect("json");
        assert!(!json.contains("tel:"), "{json}");
        assert!(json.contains("guest.emergency"), "{json}");
        assert!(json.contains(r#""type":"navigate","to":"aide""#), "{json}");
    });
}

/// Le formulaire hôte dessine ce que la configuration déclare, et une ligne par organe stocké.
#[test]
#[serial]
fn the_host_form_draws_the_rows_the_host_has() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            config_form::assert_form_matches_config(EMISSIONS, &surface, &[]);
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("shutoffs.0.kind"), "{json}");
            assert!(json.contains("shutoffs.1.location"), "{json}");
            // Deux organes stockés : pas de troisième ligne vide.
            assert!(!json.contains("shutoffs.2.kind"), "{json}");
            assert!(json.contains(r#""value":"electricity""#), "{json}");
            assert!(json.contains("Tableau électrique"), "{json}");
            // Un organe complet : rien sous la liste.
            assert!(!json.contains("Ajoutez au moins un emplacement."), "{json}");
        });
}

/// Rien de stocké : une ligne, et « Ajouter » en demande une de plus.
#[test]
#[serial]
fn an_empty_form_draws_one_row_and_can_ask_for_another() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .run(|ctx| {
            let json =
                serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
            assert!(json.contains("shutoffs.0.kind"), "{json}");
            assert!(!json.contains("shutoffs.1.kind"), "{json}");
            assert!(json.contains(r#""shutoffs_count":2"#), "{json}");
            // Le message de la spec, sous la liste — celui que la porte de publication donne.
            assert!(
                json.contains(r#""error":"Ajoutez au moins un emplacement.""#),
                "{json}"
            );
        });
}

/// La borne tient : au dernier organe, « Ajouter » n'en demande pas un de plus.
#[test]
#[serial]
fn the_bound_holds_at_the_last_row() {
    let rows: Vec<Value> = (0..safety_shutoffs::MAX_SHUTOFFS)
        .map(|i| json!({ "title": format!("Organe {i}"), "location": "Ici" }))
        .collect();
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "shutoffs": rows }))
        .run(|ctx| {
            let json =
                serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
            let last = safety_shutoffs::MAX_SHUTOFFS - 1;
            assert!(
                json.contains(&format!(r#""shutoffs.{last}.kind""#)),
                "{json}"
            );
            assert!(
                !json.contains(&format!(r#""shutoffs.{}.kind""#, last + 1)),
                "{json}"
            );
            assert!(
                json.contains(&format!(
                    r#""shutoffs_count":{}"#,
                    safety_shutoffs::MAX_SHUTOFFS
                )),
                "{json}"
            );
        });
}

/// L'enregistrement rend les deux langues : le formulaire envoie celle qu'il édite, l'autre reste.
#[test]
#[serial]
fn saving_in_english_keeps_the_french_text() {
    let stored = json!({
        "shutoffs": [{
            "id": "tableau",
            "kind": "electricity",
            "title": { "fr": "Tableau électrique", "en": "Fuse box" },
            "location": { "fr": "Placard de l’entrée", "en": "Entrance cupboard" },
            "instruction": { "fr": "Disjoncteur en haut à gauche.", "en": "Breaker, top left." }
        }],
        "general_note": { "fr": "Appelez-moi à toute heure.", "en": "Call me at any hour." }
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|mut ctx| {
            ctx.locale = "en-US".into();
            let surface = render_host_main(ctx).expect("host main");
            // L'inventaire des textes traduits : tout ce qu'un hôte qui écrit en anglais garde en
            // français. Une ligne qui disparaît d'ici est une traduction perdue au prochain
            // enregistrement.
            assert_eq!(
                config_save::localized_paths(EMISSIONS),
                [
                    "general_note",
                    "shutoffs.instruction",
                    "shutoffs.location",
                    "shutoffs.title"
                ]
            );
            let saved = config_save::save(EMISSIONS, &surface, &stored, "en");
            assert_eq!(saved["shutoffs"][0]["title"]["fr"], "Tableau électrique");
            assert_eq!(saved["shutoffs"][0]["title"]["en"], "Fuse box");
            assert_eq!(saved["shutoffs"][0]["kind"], "electricity");
            assert_eq!(saved["general_note"]["fr"], "Appelez-moi à toute heure.");
        });
}

/// La porte de publication : un organe complet suffit, une ligne à finir se signale.
#[test]
#[serial]
fn publication_needs_one_complete_shutoff() {
    let check = |config: Value| {
        MockContext::host()
            .with_capabilities(&[capability::core::STORAGE])
            .with_config(&config)
            .run(|ctx| publish_readiness(ctx).expect("readiness"))
    };

    let empty = check(json!({}));
    assert!(!empty.items[0].ok);
    assert_eq!(empty.items.len(), 1);
    assert_eq!(
        empty.items[0].hint.get("fr"),
        "Ajoutez au moins un emplacement."
    );

    let ready = check(sample_config());
    assert!(ready.items[0].ok);
    assert_eq!(ready.items.len(), 1);

    let half = check(json!({
        "shutoffs": [
            { "kind": "water", "title": "Vanne", "location": "Cave" },
            { "kind": "gas", "title": "Gaz" }
        ]
    }));
    assert!(half.items[0].ok);
    assert_eq!(half.items[1].id, "incomplete");
    assert!(!half.items[1].ok);
}

/// Avec un numéro au profil, « Appeler Claire » passe devant les numéros d'urgence (§2.22).
///
/// C'est l'action principale d'un écran qu'on lit sous le stress, et elle n'avait jamais été
/// exercée : seul le cas sans numéro l'était, à l'époque où le runtime ne servait pas encore
/// `context.host`.
#[test]
#[serial]
fn with_a_host_phone_the_call_button_comes_first() {
    guest().with_config(&sample_config()).run(|ctx| {
        let mut ctx = ctx;
        ctx.host = Some(portaki_sdk::context::HostProfile {
            name: "Claire".into(),
            phone: Some("+33612345678".into()),
            ..portaki_sdk::context::HostProfile::default()
        });
        let json =
            serde_json::to_string(&render_explore_detail(ctx).expect("detail")).expect("json");

        assert!(json.contains("tel:+33612345678"), "{json}");
        // Le libellé nommé, pas le repli sans nom : `t!` rend la clé telle quelle sous un bundle
        // de test vide, mais c'est bien la branche « avec nom » qui a été prise.
        assert!(json.contains("guest.call\""), "{json}");
        assert!(!json.contains("guest.call.plain"), "{json}");
        // « Le mot de Claire », pas « Le mot de votre hôte ».
        assert!(json.contains("guest.note.title.named"), "{json}");
        assert!(!json.contains("i18n:guest.note.title"), "{json}");
        // L'appel d'abord, les numéros d'urgence ensuite : on compose avant de chercher.
        let call = json.find("tel:+33612345678").expect("appel");
        let emergency = json.find("guest.emergency").expect("urgences");
        assert!(call < emergency, "{json}");
    });
}
