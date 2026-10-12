//! L'onglet « Calendrier des événements » (planche du 12 oct.).

use portaki_sdk::capability;
use portaki_test_utils::MockContext;
use serde_json::{json, Value};
use serial_test::serial;

use events::{render_explore_detail, render_host_calendar};

/// « Festival du port » importé le 28 juillet 2099 à 19:00 UTC, quai de Cannes.
fn openagenda_payload() -> String {
    json!({
        "total": 1,
        "events": [{
            "uid": 56158955,
            "title": "Festival du port",
            "location": { "name": "Quai", "city": "Cannes", "latitude": 43.55, "longitude": 7.01 },
            "nextTiming": { "begin": "2099-07-28T19:00:00.000Z" },
            "canonicalUrl": "https://openagenda.com/events/festival-du-port"
        }]
    })
    .to_string()
}

/// Le même festival saisi par l'hôte, une demi-heure plus tard, au même endroit ; et un marché
/// chaque mardi depuis le 1er juillet.
fn config() -> Value {
    json!({
        "nearby_enabled": true,
        "events": [
            { "id": "fest", "title": { "fr": "Festival du port" }, "starts_at": "2099-07-28T19:30:00Z",
              "lat": 43.5501, "lng": 7.0101 },
            { "id": "market", "title": { "fr": "Marché du port" }, "starts_at": "2099-07-01T08:00:00Z",
              "recurrence": "weekly", "weekdays": ["tue"], "lat": 43.55, "lng": 7.0 }
        ]
    })
}

fn host(now: &str) -> portaki_test_utils::MockContextBuilder {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE, capability::external::OPEN_AGENDA_POOL])
        .with_connector_response("open-agenda", "nearby_events", openagenda_payload())
        .with_now(now.parse().unwrap())
        .with_config(&config())
        .with_translation("host.calendar.quoted", "« {name} »")
        .with_translation(
            "host.calendar.duplicates.one",
            "{count} doublon : {names} est saisi et importé d’OpenAgenda. Le vôtre est gardé dans le livret.",
        )
}

fn render(now: &str, input: Value) -> Value {
    host(now).run(|mut ctx| {
        ctx.input = input;
        serde_json::to_value(render_host_calendar(ctx).expect("calendar")).unwrap()
    })
}

fn find<'a>(value: &'a Value, kind: &str) -> Vec<&'a Value> {
    let mut found = Vec::new();
    fn walk<'a>(value: &'a Value, kind: &str, found: &mut Vec<&'a Value>) {
        match value {
            Value::Object(map) => {
                if map.get("type").and_then(Value::as_str) == Some(kind) {
                    found.push(value);
                }
                map.values().for_each(|v| walk(v, kind, found));
            }
            Value::Array(items) => items.iter().for_each(|v| walk(v, kind, found)),
            _ => {}
        }
    }
    walk(value, kind, &mut found);
    found
}

/// Les pastilles, jour par jour : `(date, libellé, ton)`.
fn chips(surface: &Value) -> Vec<(String, String, String)> {
    let calendar = find(surface, "MonthCalendar")[0];
    calendar["days"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|day| {
            day["items"].as_array().unwrap().iter().map(move |item| {
                (
                    day["date"].as_str().unwrap().to_string(),
                    item["label"].as_str().unwrap().to_string(),
                    item["tone"].as_str().unwrap_or("").to_string(),
                )
            })
        })
        .collect()
}

#[test]
#[serial]
fn the_month_spreads_recurrences_and_flags_the_duplicate() {
    let surface = render("2099-07-02T10:00:00Z", json!({}));
    let calendar = find(&surface, "MonthCalendar")[0];
    assert_eq!(calendar["month"], "2099-07");
    assert_eq!(calendar["nextAction"]["payload"]["month"], "2099-08");
    assert_eq!(calendar["prevAction"]["payload"]["month"], "2099-06");

    let all = chips(&surface);
    let markets: Vec<&str> = all
        .iter()
        .filter(|(_, label, _)| label == "Marché du port")
        .map(|(date, _, _)| date.as_str())
        .collect();
    assert_eq!(
        markets,
        ["2099-07-07", "2099-07-14", "2099-07-21", "2099-07-28"]
    );
    // Le doublon : le vôtre seul, en « Doublon » ; l'import ne s'affiche pas en double.
    let festival: Vec<_> = all
        .iter()
        .filter(|(_, l, _)| l == "Festival du port")
        .collect();
    assert_eq!(festival.len(), 1, "{all:?}");
    assert_eq!(festival[0].2, "warning");

    let notice = find(&surface, "InlineNotice");
    assert_eq!(notice.len(), 1);
    assert_eq!(
        notice[0]["message"],
        "1 doublon : « Festival du port » est saisi et importé d’OpenAgenda. Le vôtre est gardé dans le livret."
    );
}

#[test]
#[serial]
fn the_source_filter_and_the_thirty_days() {
    // Filtre OpenAgenda : l'import seul, en « Doublon » lui aussi.
    let imported = chips(&render(
        "2099-07-02T10:00:00Z",
        json!({ "source": "openagenda" }),
    ));
    assert_eq!(
        imported,
        [(
            "2099-07-28".into(),
            "Festival du port".into(),
            "warning".into()
        )]
    );

    // Le 20 août à 7 h, juillet ne garde que ses trente derniers jours, grisés : le mardi 21 à
    // 8 h y est encore, le 14 non.
    let surface = render(
        "2099-08-20T07:00:00Z",
        json!({ "month": "2099-07", "source": "own" }),
    );
    let calendar = find(&surface, "MonthCalendar")[0];
    let days: Vec<&str> = calendar["days"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["date"].as_str().unwrap())
        .collect();
    assert_eq!(days, ["2099-07-21", "2099-07-28"]);
    assert!(calendar["days"][0]["items"][0]["muted"].as_bool().unwrap());
}

#[test]
#[serial]
fn a_click_opens_the_event_and_yours_leads_to_the_settings() {
    let surface = render(
        "2099-07-02T10:00:00Z",
        json!({ "selected": "fest@2099-07-28" }),
    );
    let cards = find(&surface, "Card");
    let detail = cards
        .iter()
        .find(|c| c["title"] == "Festival du port")
        .expect("detail");
    let button = &find(detail, "Button")[0];
    assert_eq!(button["action"]["type"], "navigate");
    let to = button["action"]["to"].as_str().unwrap();
    assert!(
        to.starts_with("/listings/") && to.contains("/modules/"),
        "{to}"
    );

    let surface = render("2099-07-02T10:00:00Z", json!({ "selected": "oa-56158955" }));
    let cards = find(&surface, "Card");
    let detail = cards
        .iter()
        .find(|c| c["title"] == "Festival du port")
        .expect("detail");
    let button = &find(detail, "Button")[0];
    assert_eq!(button["action"]["type"], "external");
}

/// §9 #6 : le flux injoignable — vos événements seuls, et un avertissement.
#[test]
#[serial]
fn an_unreachable_agenda_is_said() {
    let surface = MockContext::host()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::external::OPEN_AGENDA_POOL,
        ])
        .with_connector_error("open-agenda", "nearby_events", "timeout")
        .with_now("2099-07-02T10:00:00Z".parse().unwrap())
        .with_config(&config())
        .run(|ctx| serde_json::to_value(render_host_calendar(ctx).expect("calendar")).unwrap());
    let notices = find(&surface, "InlineNotice");
    assert!(notices
        .iter()
        .any(|n| n["message"] == "i18n:host.calendar.unreachable"));
    assert!(chips(&surface)
        .iter()
        .any(|(_, l, t)| l == "Festival du port" && t == "neutral"));
}

/// Le livret applique la même règle que le bandeau : le doublon ne s'affiche qu'une fois.
#[test]
#[serial]
fn the_booklet_keeps_yours_only() {
    MockContext::guest()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::external::OPEN_AGENDA_POOL,
        ])
        .with_connector_response("open-agenda", "nearby_events", openagenda_payload())
        .with_now("2099-07-02T10:00:00Z".parse().unwrap())
        .with_config(&config())
        .run(|ctx| {
            let json = serde_json::to_string(&render_explore_detail(ctx).expect("detail")).unwrap();
            assert!(json.contains("events/detail/fest"), "{json}");
            assert!(
                !json.contains("56158955"),
                "l'import en double est écarté : {json}"
            );
        });
}

/// Un marché chaque mardi, importé aussi au deuxième mardi du séjour : le livret ne garde que le
/// vôtre, comme le bandeau de l'onglet le promet.
#[test]
#[serial]
fn the_booklet_drops_an_import_of_a_later_recurrence() {
    use portaki_sdk::context::StayContext;
    let market = json!({
        "total": 1,
        "events": [{
            "uid": 77,
            "title": "Marché du port",
            "location": { "name": "Port", "city": "Cannes", "latitude": 43.55, "longitude": 7.0 },
            "nextTiming": { "begin": "2099-07-14T08:30:00.000Z" }
        }]
    })
    .to_string();
    MockContext::guest()
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::external::OPEN_AGENDA_POOL,
        ])
        .with_connector_response("open-agenda", "nearby_events", market)
        .with_now("2099-07-05T10:00:00Z".parse().unwrap())
        .with_stay(StayContext {
            checkin_at: Some("2099-07-06T15:00:00Z".parse().unwrap()),
            checkout_at: Some("2099-07-16T10:00:00Z".parse().unwrap()),
            ..StayContext::default()
        })
        .with_config(&config())
        .run(|ctx| {
            let json = serde_json::to_string(&render_explore_detail(ctx).expect("detail")).unwrap();
            assert!(json.contains("events/detail/market"), "{json}");
            assert!(
                !json.contains("oa-77"),
                "l'import du deuxième mardi est écarté : {json}"
            );
        });
}
