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

/// Le tableau (§2.2) : une ligne par événement saisi, triée par date, chacune avec son panneau
/// latéral ; « Ajouter » en ouvre une, avec son id, jusqu'à cinquante.
#[test]
#[serial]
fn the_table_lists_the_events_sorted_and_adds_one() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({}))
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            let table = data_table(&surface);
            assert_eq!(table["panel"], "sheet");
            assert!(
                table.get("sortable").is_none(),
                "trié par date, pas à la main"
            );
            assert_eq!(table["rows"].as_array().unwrap().len(), 0);
            let added = table["addAction"]["payload"]["newEvents"]
                .as_str()
                .expect("newEvents");
            assert_eq!(added.len(), 36, "{added}");
        });

    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_now("2099-07-01T10:00:00Z".parse().unwrap())
        .with_config(&json!({ "events": [
            { "id": "tard", "title": "Concert", "starts_at": "2099-08-22T20:00:00Z", "price": "12 €" },
            { "id": "passe", "title": "Fête du jardin", "starts_at": "2099-06-02T10:00:00Z" },
            { "title": "", "place": "", "starts_at": "" },
            { "id": "tot", "title": "Feu d'artifice", "date": "2099-08-15", "start_time": "22:30" }
        ] }))
        .run(|mut ctx| {
            ctx.input = json!({ "newEvents": "nouveau" });
            let surface = render_host_main(ctx).expect("host main");
            let table = data_table(&surface);
            let ids: Vec<&str> = table["rows"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| row["id"].as_str().unwrap())
                .collect();
            // À venir par date, puis le passé, puis l'ajouté ; la ligne vide n'est plus là.
            assert_eq!(ids, ["tot", "tard", "passe", "nouveau"]);
            let rows = table["rows"].as_array().unwrap();
            assert_eq!(rows[0]["when"], "15/08 · 22:30");
            assert_eq!(rows[1]["price"], "12 €");
            assert_eq!(rows[2]["badge"], "i18n:host.event.badge.past");
            // Le panneau du rang 0 est celui de « tot », nommé à son rang affiché.
            let panel = serde_json::to_string(&table["children"][0]).unwrap();
            assert!(panel.contains(r#""name":"events.0.date""#), "{panel}");
            assert!(panel.contains("2099-08-15"), "{panel}");
            let json = serde_json::to_string(&surface).unwrap();
            assert!(json.contains("4 / 50"), "{json}");
        });

    let rows: Vec<Value> = (0..events::MAX_EVENTS)
        .map(
            |i| json!({ "id": format!("e{i}"), "title": format!("Événement {i}"), "place": "Ici" }),
        )
        .collect();
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "events": rows }))
        .run(|ctx| {
            let surface = render_host_main(ctx).expect("host main");
            let table = data_table(&surface);
            assert_eq!(table["rows"].as_array().unwrap().len(), events::MAX_EVENTS);
            // La borne tient : au cinquantième, plus d'« Ajouter ».
            assert!(table.get("addAction").is_none());
            let json = serde_json::to_string(&surface).unwrap();
            assert!(json.contains("i18n:host.events.tooMany"), "{json}");
        });
}

/// Le `DataTable` d'une surface.
fn data_table(surface: &portaki_sdk::sdui::surface::Surface) -> Value {
    fn find(value: &Value) -> Option<Value> {
        match value {
            Value::Object(object) if object.get("type") == Some(&json!("DataTable")) => {
                Some(value.clone())
            }
            Value::Object(object) => object.values().find_map(find),
            Value::Array(items) => items.iter().find_map(find),
            _ => None,
        }
    }
    find(&serde_json::to_value(surface).unwrap()).expect("a DataTable")
}

/// La surface telle que le tableau de bord l'enregistre : chaque ligne du tableau envoie son id
/// (`events.<i>.id`), ce que `config_save` ne sait pas lire d'un `DataTable`.
fn with_row_ids(
    surface: &portaki_sdk::sdui::surface::Surface,
) -> portaki_sdk::sdui::surface::Surface {
    fn walk(value: &mut Value) {
        match value {
            Value::Object(object) if object.get("type") == Some(&json!("DataTable")) => {
                let name = object["rowName"].as_str().unwrap_or_default().to_string();
                let ids: Vec<Value> = object["rows"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|row| row["id"].clone())
                    .collect();
                let panels = object["children"].as_array_mut().unwrap();
                for (index, (panel, id)) in panels.iter_mut().zip(ids).enumerate() {
                    panel["children"].as_array_mut().unwrap().push(json!({
                        "type": "TextInput", "name": format!("{name}.{index}.id"), "value": id
                    }));
                }
            }
            Value::Object(object) => object.values_mut().for_each(walk),
            Value::Array(items) => items.iter_mut().for_each(walk),
            _ => {}
        }
    }
    let mut value = serde_json::to_value(surface).unwrap();
    walk(&mut value);
    serde_json::from_value(value).unwrap()
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
            let surface = with_row_ids(&render_host_main(ctx).expect("host main"));
            let sent = config_save::form_args(&surface);
            // Une ligne par événement, triées par date ; la ligne vide n'est plus dessinée.
            assert_eq!(sent["events"][0]["id"], "evt-1");
            assert_eq!(sent["events"][0]["title"], "Jazz concert");
            assert_eq!(sent["events"][1]["id"], "evt-3");
            assert_eq!(sent["events"][1]["title"], "Brocante");
            assert_eq!(sent["events"][1]["date"], "2099-07-26");
            assert_eq!(sent["events"][1]["start_time"], "09:00");
            assert_eq!(sent["events"].as_array().unwrap().len(), 2);
            assert_eq!(sent["disclaimer"], "Dates are indicative");

            let saved = config_save::save(EMISSIONS, &surface, &stored, "en");
            let first = &saved["events"][0];
            assert_eq!(first["title"], stored["events"][0]["title"]);
            assert_eq!(first["place"], stored["events"][0]["place"]);
            assert_eq!(first["ends_at"], stored["events"][0]["ends_at"]);
            assert_eq!(first["note"], stored["events"][0]["note"]);
            assert_eq!(saved["events"][1]["title"]["fr"], "Brocante");
            assert_eq!(saved["events"][1]["place"]["fr"], "Port");
            // La description est désormais dans le formulaire : un enregistrement en anglais y
            // écrit l'anglais, et le français reste. Avant, elle n'était portée par aucun champ
            // et traversait les sauvegardes sans changer.
            assert_eq!(saved["events"][1]["note"]["fr"], "Gratuit");
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
/// date » dessine une ligne de plus sur cet événement seul. Chacun ne se montre qu'à sa
/// répétition (`visibleWhen`).
#[test]
#[serial]
fn the_form_draws_weekdays_and_dates() {
    let config = json!({
        "events": [
            { "id": "marche", "title": "Marché", "recurrence": "weekly", "weekdays": ["tue", "thu"],
              "date": "2099-07-07", "start_time": "08:00" },
            { "id": "festival", "title": "Festival", "recurrence": "dates",
              "date": "2099-07-10", "dates": [{ "date": "2099-08-14" }] }
        ]
    });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_now("2099-07-01T10:00:00Z".parse().unwrap())
        .with_config(&config)
        .run(|ctx| {
            let json =
                serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
            assert!(json.contains("events.0.weekdays"), "{json}");
            assert!(json.contains(r#"[\"tue\",\"thu\"]"#), "{json}");
            assert!(
                json.contains(r#""visibleWhen":"events.0.recurrence=weekly""#),
                "{json}"
            );
            assert!(
                json.contains(r#""visibleWhen":"events.1.recurrence=dates""#),
                "{json}"
            );
            assert!(json.contains(r#""layout":"segmented""#), "{json}");
            assert!(json.contains("events.1.dates.0.date"), "{json}");
            assert!(json.contains("2099-08-14"), "{json}");
            assert!(!json.contains("events.1.dates.1.date"), "{json}");
            assert!(json.contains(r#""dates.festival":2"#), "{json}");
        });
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_now("2099-07-01T10:00:00Z".parse().unwrap())
        .with_config(&config)
        .run(|mut ctx| {
            ctx.input = json!({ "dates.festival": 2 });
            let json =
                serde_json::to_string(&render_host_main(ctx).expect("host main")).expect("json");
            assert!(json.contains("events.1.dates.1.date"), "{json}");
            assert!(!json.contains("events.0.dates.0.date"), "{json}");
        });
}

/// « Événements consultés » : les fiches d'événement ouvertes sur la période ; « — » sans
/// statistiques du livret, « 0 » quand personne n'a rien ouvert.
#[test]
fn the_stats_tile_counts_the_events_viewed() {
    let tile = |views: serde_json::Value| {
        let mut args = json!({
            "propertyId": "00000000-0000-0000-0000-000000000000", "period": 30, "key": "events-views"
        });
        if !views.is_null() {
            args["views"] = views;
        }
        let mut value = String::new();
        MockContext::host().run(|ctx| {
            value = events::stats_summary(ctx, serde_json::from_value(args).expect("args"))
                .expect("summary")
                .value;
        });
        value
    };
    assert_eq!(tile(serde_json::Value::Null), "—");
    assert_eq!(tile(json!({ "opens": 0 })), "0");
    assert_eq!(
        tile(json!({ "opens": 9, "surfaces": { "explore.item": 7, "explore.detail": 2 } })),
        "7"
    );

    let detail = |input: serde_json::Value| {
        let mut out = String::new();
        MockContext::host().run(|mut ctx| {
            ctx.input = input;
            out = serde_json::to_string(&events::render_host_stats(ctx)).expect("json");
        });
        out
    };
    assert!(detail(json!({ "periodDays": 30 })).contains("stats.views.unavailable"));
    assert!(detail(json!({ "views": { "opens": 0 } })).contains("stats.views.empty"));
    let full = detail(json!({ "periodDays": 90, "views": {
        "opens": 9, "surfaces": { "explore.item": 7, "explore.detail": 2 }, "clicks": { "directions": 3 }
    } }));
    assert!(full.contains(r#""value":"7""#), "{full}");
    assert!(full.contains("stats.views.clicks.directions"), "{full}");
    assert!(!full.contains("stats.views.clicks.call"), "{full}");
}
