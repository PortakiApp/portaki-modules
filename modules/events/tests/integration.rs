//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use events::{
    render_explore_detail, render_explore_item, render_home_card, render_host_main,
    render_upcoming_card,
};
use portaki_test_utils::{MockContext, Property, SurfaceAssertions};
use serde_json::{json, Value};

#[path = "../../../support/config_form.rs"]
mod config_form;
#[path = "../../../support/config_save.rs"]
mod config_save;

const EMISSIONS: &str = concat!(env!("OUT_DIR"), "/portaki-emissions");

fn sample_config() -> serde_json::Value {
    json!({
        "events": [{
            "id": "evt-1",
            "title": {"fr": "Concert jazz", "en": "Jazz concert"},
            "place": {"fr": "Théâtre de la Mer", "en": "Sea theatre"},
            "starts_at": "2099-07-25T18:00:00Z",
            "url": "https://example.com/tickets",
            "lat": 43.58,
            "lng": 7.12,
            "note": {"fr": "Arrivez tôt.", "en": "Arrive early."}
        }],
        "disclaimer": "Dates indicatives",
        "nearby_enabled": false
    })
}

fn openagenda_payload() -> String {
    json!({
        "total": 1,
        "events": [{
            "uid": 56158955,
            "title": "Festival du port",
            "location": {
                "name": "Quai",
                "city": "Cannes",
                "latitude": 43.55,
                "longitude": 7.01
            },
            "nextTiming": { "begin": "2099-07-28T19:00:00.000Z" },
            "canonicalUrl": "https://openagenda.com/events/festival-du-port"
        }]
    })
    .to_string()
}

#[test]
#[serial]
fn home_card_empty_without_config() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            assert!(
                SurfaceAssertions::new(&render_home_card(ctx).expect("surface"))
                    .contains_type("EmptyState")
            );
        });
}

#[test]
#[serial]
fn home_card_headlines_the_next_event() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            // L'événement qui arrive est en vedette, pas en ligne : heure en pastille, titre en
            // grand, lieu dessous (§2.11).
            assert!(SurfaceAssertions::new(&surface).contains_type("Badge"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("Concert jazz"));
            assert!(json.contains("Th\u{e9}\u{e2}tre de la Mer"));
            // Un seul événement : pas de ligne sous la vedette, et pas d'intertitre « Ensuite ».
            assert!(!SurfaceAssertions::new(&surface).contains_type("ListItem"));
            assert!(!SurfaceAssertions::new(&surface).contains_type("Eyebrow"));
            assert!(json.contains("bottomSheet"));
        });
}

#[test]
#[serial]
fn upcoming_card_is_compact_with_next_event_headline() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_upcoming_card(ctx).expect("surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Text"));
            // Compact: no full event list / map on the prep card.
            assert!(!SurfaceAssertions::new(&surface).contains_type("ListItem"));
            assert!(!SurfaceAssertions::new(&surface).contains_type("Map"));

            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("upcoming.card"));
            assert!(json.contains("Concert jazz") || json.contains("Jazz concert"));
        });
}

#[test]
#[serial]
fn upcoming_card_empty_without_config() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            assert!(
                SurfaceAssertions::new(&render_upcoming_card(ctx).expect("surface"))
                    .contains_type("EmptyState")
            );
        });
}

#[test]
#[serial]
fn detail_includes_map_and_link() {
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&sample_config())
        .run(|ctx| {
            let surface = render_explore_detail(ctx).expect("surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Map"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Link"));
            assert!(SurfaceAssertions::new(&surface).contains_type("InfoBanner"));
        });
}

#[test]
#[serial]
fn home_card_renders_openagenda_nearby() {
    MockContext::guest()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::external::OPEN_AGENDA_POOL,
        ])
        .with_connector_response("open-agenda", "nearby_events", openagenda_payload())
        .with_config(&json!({
            "nearby_enabled": true,
            "radius_km": 40
        }))
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("surface");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Badge"));
            let json = serde_json::to_string(&surface).expect("json");
            assert!(json.contains("Festival du port"));
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
            config_form::assert_form_matches_config(EMISSIONS, &surface, &[]);
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("Concert jazz"));
            assert!(json.contains("Dates indicatives"));
        });
}

/// A property not geocoded yet: no nearby search, no call, and a guest empty state.
#[test]
#[serial]
fn without_a_position_nothing_is_searched_nearby() {
    MockContext::guest()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::external::OPEN_AGENDA_POOL,
        ])
        .with_coordinates(None)
        .with_connector_response("open-agenda", "nearby_events", openagenda_payload())
        .with_config(&json!({ "nearby_enabled": true }))
        .run_with(|ctx, host| {
            let detail = render_explore_detail(ctx.clone()).expect("surface");
            assert!(SurfaceAssertions::new(&detail).contains_type("EmptyState"));
            let json = serde_json::to_string(&detail).expect("json");
            assert!(json.contains("i18n:guest.empty.title"), "{json}");
            assert!(render_home_card(ctx)
                .map(|s| SurfaceAssertions::new(&s).contains_type("EmptyState"))
                .expect("surface"));
            assert!(host.connector_calls().is_empty());
        });
}

/// Les lignes suivent ce que l'hôte a saisi, et « Ajouter » en demande une de plus.
///
/// Six emplacements figés gelaient la liste à six : l'hôte voyait quatre cartes vides quand il
/// avait saisi deux événements, et ne pouvait pas en saisir un septième parce que le formulaire
/// ne le dessinait jamais.
#[test]
#[serial]
fn the_form_draws_the_rows_the_host_has() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .run(|ctx| {
            let json =
                serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
            assert!(json.contains("events.0.title"), "{json}");
            assert!(!json.contains("events.1.title"), "{json}");
            assert!(json.contains(r#""events_count":2"#), "{json}");
        });

    let rows: Vec<Value> = (0..events::MAX_EVENTS)
        .map(|i| json!({ "title": format!("Événement {i}"), "place": "Ici" }))
        .collect();
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "events": rows }))
        .run(|ctx| {
            let json =
                serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
            let last = events::MAX_EVENTS - 1;
            assert!(json.contains(&format!("events.{last}.title")), "{json}");
            assert!(
                !json.contains(&format!("events.{}.title", last + 1)),
                "{json}"
            );
            // La borne tient : au dernier, « Ajouter » n'en demande pas un de plus.
            assert!(
                json.contains(&format!(r#""events_count":{}"#, events::MAX_EVENTS)),
                "{json}"
            );
        });
}

/// « Accès » est sa propre carte, et elle passe avant « Bon à savoir » (§2.11).
///
/// Sa propre carte parce qu'un conseil se lit s'il reste du temps et l'accès se lit avant de
/// partir ; avant, parce qu'un voyageur en fauteuil ne doit pas la chercher au milieu des bons
/// plans de parking. Sans accès saisi, pas de carte vide.
#[test]
#[serial]
fn access_is_its_own_card_before_the_tips() {
    let event = |extra: serde_json::Value| {
        let mut row = json!({
            "id": "marche",
            "title": { "fr": "Marché nocturne" },
            "place": { "fr": "Place du village" },
            "starts_at": "2026-06-02T18:00:00",
            "tips": { "fr": "Venez avant 19 h." }
        });
        if let (Some(row), Some(extra)) = (row.as_object_mut(), extra.as_object()) {
            for (key, value) in extra {
                row.insert(key.clone(), value.clone());
            }
        }
        json!({ "events": [row] })
    };

    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_property(Property::default())
        .with_config(&event(json!({
            "access": { "fr": "Place piétonne, accès de plain-pied" }
        })))
        .run(|mut ctx| {
            ctx.input = json!({ "eventId": "marche" });
            let json_text =
                serde_json::to_string(&render_explore_item(ctx).expect("fiche")).unwrap();
            assert!(json_text.contains("i18n:guest.access"), "{json_text}");
            assert!(json_text.contains("Place piétonne, accès de plain-pied"));
            // L'accès avant les conseils, pas après.
            let access = json_text.find("i18n:guest.access").expect("accès");
            let tips = json_text.find("i18n:guest.goodToKnow").expect("conseils");
            assert!(access < tips, "l'accès passe avant « Bon à savoir »");
        });

    // Sans accès saisi, aucune carte : une carte vide promettrait une information qui n'existe pas.
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_property(Property::default())
        .with_config(&event(json!({})))
        .run(|mut ctx| {
            ctx.input = json!({ "eventId": "marche" });
            let json_text =
                serde_json::to_string(&render_explore_item(ctx).expect("fiche")).unwrap();
            assert!(!json_text.contains("i18n:guest.access"), "{json_text}");
            assert!(json_text.contains("i18n:guest.goodToKnow"));
        });
}

/// A host writing in English: the French texts stay, and so does the id the form does not carry;
/// rows keep their place.
#[test]
#[serial]
fn a_save_in_english_keeps_the_french() {
    assert_eq!(
        config_save::localized_paths(EMISSIONS),
        [
            "disclaimer",
            "events.access",
            "events.note",
            "events.place",
            "events.tips",
            "events.title"
        ]
    );
    let stored = json!({
        "events": [
            { "id": "evt-1", "title": { "fr": "Concert jazz", "en": "Jazz concert" },
              "place": { "fr": "Théâtre de la Mer", "en": "Sea theatre" },
              "starts_at": "2099-07-25T18:00:00Z", "ends_at": "2099-07-25T20:00:00Z",
              "url": "https://example.com/tickets", "lat": 43.58, "lng": 7.12,
              "note": { "fr": "Arrivez tôt.", "en": "Arrive early." } },
            { "title": "", "place": "", "starts_at": "", "url": "", "lat": "", "lng": "" },
            { "id": "evt-3", "title": { "fr": "Brocante" }, "place": { "fr": "Port" },
              "starts_at": "2099-07-26T09:00:00Z", "note": { "fr": "Gratuit" } }
        ],
        "disclaimer": { "fr": "Dates indicatives", "en": "Dates are indicative" },
        "nearby_enabled": false,
        "radius_km": "40"
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&stored)
        .run(|mut ctx| {
            ctx.locale = "en-US".into();
            let surface = render_host_main(ctx).expect("host main");
            let sent = config_save::form_args(&surface);
            // Stored order, the blank row where it was; ids on the filled rows only.
            assert_eq!(sent["events"][0]["id"], "evt-1");
            assert_eq!(sent["events"][0]["title"], "Jazz concert");
            assert!(sent["events"][1].get("id").is_none());
            assert_eq!(sent["events"][2]["id"], "evt-3");
            assert_eq!(sent["events"][2]["title"], "Brocante");
            // Trois lignes stockées, trois dessinées : plus de créneaux vides en bout de liste.
            assert_eq!(sent["events"].as_array().unwrap().len(), 3);
            assert_eq!(sent["disclaimer"], "Dates are indicative");

            let saved = config_save::save(EMISSIONS, &surface, &stored, "en");
            let first = &saved["events"][0];
            assert_eq!(first["title"], stored["events"][0]["title"]);
            assert_eq!(first["place"], stored["events"][0]["place"]);
            assert_eq!(first["ends_at"], stored["events"][0]["ends_at"]);
            assert_eq!(first["note"], stored["events"][0]["note"]);
            assert_eq!(saved["events"][2]["title"]["fr"], "Brocante");
            assert_eq!(saved["events"][2]["place"]["fr"], "Port");
            // La description est désormais dans le formulaire : un enregistrement en anglais y
            // écrit l'anglais, et le français reste. Avant, elle n'était portée par aucun champ
            // et traversait les sauvegardes sans changer.
            assert_eq!(saved["events"][2]["note"]["fr"], "Gratuit");
            assert_eq!(saved["disclaimer"], stored["disclaimer"]);
        });
}

/// §9 #1 : l'hôte a un événement, mais hors du séjour — le voyageur lit l'état vide, pas une
/// liste vide.
#[test]
#[serial]
fn an_event_outside_the_stay_shows_the_empty_state() {
    use portaki_sdk::context::StayContext;
    let stay = StayContext {
        checkin_at: Some("2099-08-10T15:00:00Z".parse().unwrap()),
        checkout_at: Some("2099-08-13T10:00:00Z".parse().unwrap()),
        ..StayContext::default()
    };
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_stay(stay)
        .with_config(&sample_config())
        .run(|ctx| {
            let detail = render_explore_detail(ctx).expect("surface");
            assert!(SurfaceAssertions::new(&detail).contains_type("EmptyState"));
            let json = serde_json::to_string(&detail).expect("json");
            assert!(json.contains("i18n:guest.empty.title"), "{json}");
        });
}

/// §9 #2 : un événement ce soir, à l'heure du logement, porte le badge « Ce soir ».
#[test]
#[serial]
fn tonight_event_carries_the_badge() {
    let config = json!({
        "nearby_enabled": false,
        "events": [{ "id": "evt-1", "title": "Feu d'artifice", "starts_at": "2099-07-14T20:00:00Z" }]
    });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_property(Property::default())
        .with_now("2099-07-14T10:00:00Z".parse().unwrap())
        .with_config(&config)
        .run(|ctx| {
            let home = serde_json::to_string(&render_home_card(ctx.clone()).unwrap()).unwrap();
            assert!(home.contains("i18n:guest.event.tonight"), "{home}");
            let detail = serde_json::to_string(&render_explore_detail(ctx).unwrap()).unwrap();
            assert!(detail.contains("i18n:guest.event.tonight"), "{detail}");
        });
    MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_property(Property::default())
        .with_now("2099-07-13T10:00:00Z".parse().unwrap())
        .with_config(&config)
        .run(|ctx| {
            let home = serde_json::to_string(&render_home_card(ctx).unwrap()).unwrap();
            assert!(!home.contains("i18n:guest.event.tonight"), "{home}");
        });
}

/// « Tous les mardis et jeudis » sur la fiche d'un événement hebdomadaire (§2.2).
#[test]
#[serial]
fn a_weekly_event_names_its_days_on_its_page() {
    let bundle: std::collections::BTreeMap<String, String> =
        serde_json::from_str(include_str!("../i18n/fr-FR.json")).unwrap();
    let mut mock = MockContext::guest()
        .with_capabilities(&[capability::core::STORAGE])
        .with_property(Property::default())
        .with_config(&json!({
            "nearby_enabled": false,
            "events": [{
                "id": "marche", "title": { "fr": "Marché" }, "starts_at": "2099-07-07T08:00:00Z",
                "recurrence": "weekly", "weekdays": ["thu", "tue"]
            }]
        }));
    for (key, value) in bundle {
        mock = mock.with_translation(key, value);
    }
    mock.run(|mut ctx| {
        ctx.input = json!({ "eventId": "marche" });
        let json_text = serde_json::to_string(&render_explore_item(ctx).expect("fiche")).unwrap();
        assert!(
            json_text.contains("Tous les mardis et jeudis"),
            "{json_text}"
        );
    });
}

/// Les jours en choix multiple, les dates en liste : `events.0.dates.1.date`, et « Ajouter une
/// date » dessine une ligne de plus sur cet événement seul.
#[test]
#[serial]
fn the_form_draws_weekdays_and_dates() {
    let config = json!({
        "events": [
            { "title": "Marché", "recurrence": "weekly", "weekdays": ["tue", "thu"] },
            { "title": "Festival", "recurrence": "dates", "dates": [{ "date": "2026-08-14" }] }
        ]
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .run(|ctx| {
            let json =
                serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
            assert!(json.contains("events.0.weekdays"), "{json}");
            assert!(json.contains(r#"[\"tue\",\"thu\"]"#), "{json}");
            assert!(
                json.contains("i18n:host.event.recurrence.label.dates"),
                "{json}"
            );
            assert!(json.contains("events.1.dates.0.date"), "{json}");
            assert!(json.contains("2026-08-14"), "{json}");
            assert!(!json.contains("events.1.dates.1.date"), "{json}");
            assert!(json.contains(r#""dates_for":1"#), "{json}");
            assert!(json.contains(r#""dates_count":2"#), "{json}");
        });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&config)
        .run(|mut ctx| {
            ctx.input = json!({ "events_count": 2, "dates_for": 1, "dates_count": 2 });
            let json =
                serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
            assert!(json.contains("events.1.dates.1.date"), "{json}");
            assert!(!json.contains("events.0.dates.0.date"), "{json}");
        });
}
