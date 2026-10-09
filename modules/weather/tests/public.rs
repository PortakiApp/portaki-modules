//! Bloc Climat de la page publique (`property.public`), rendu pour un visiteur sans séjour.

use chrono::{TimeZone, Utc};
use portaki_sdk::capability;
use portaki_sdk::host::runtime::HostBackend;
use portaki_sdk::sdui::common::GeoPoint;
use portaki_sdk::surfaces::check_property_public_tree;
use portaki_test_utils::MockContext;
use serde_json::{json, Value};
use serial_test::serial;

use weather::{render_property_public, reset_test_harness};

const CANNES: GeoPoint = GeoPoint {
    lat: 43.55134,
    lng: 7.01276,
};

/// Mi-juin : la saison va d'avril à septembre.
const SEASON: [(u32, &str); 6] = [
    (4, "apr"),
    (5, "may"),
    (6, "jun"),
    (7, "jul"),
    (8, "aug"),
    (9, "sep"),
];

fn current_json() -> String {
    json!({ "name": "Cannes", "main": { "temp": 23.4, "humidity": 58 }, "weather": [{ "main": "Clear" }] })
        .to_string()
}

fn forecast_json() -> String {
    json!({ "city": { "name": "Cannes" }, "list": [] }).to_string()
}

/// La réponse de `aggregated/month` : des kelvins.
fn month_json() -> String {
    json!({ "cod": 200, "result": { "month": 6, "temp": { "mean": 295.15, "median": 295.0 } } })
        .to_string()
}

fn month_key(month: u32) -> String {
    format!("climate.43.55.7.01.{month}")
}

fn visitor(config: Value) -> MockContext {
    // `with_capabilities` rebuilds the context: the public visitor would be lost.
    MockContext::public_visitor()
        .with_extra_capability_ids(&[capability::external::OPEN_WEATHER_POOL.as_str()])
        .with_coordinates(Some(CANNES))
        .with_config(&config)
        .with_now(Utc.with_ymd_and_hms(2026, 6, 15, 10, 0, 0).unwrap())
        .with_connector_response("open-weather", "current", current_json())
        .with_connector_response("open-weather", "forecast", forecast_json())
}

fn enabled() -> Value {
    json!({ "public_enabled": true })
}

fn with_season_cached(mut mock: MockContext) -> MockContext {
    for (month, _) in SEASON {
        mock = mock.with_kv(month_key(month), b"18.5".to_vec());
    }
    mock
}

fn root(surface: &portaki_sdk::sdui::surface::Surface) -> Value {
    serde_json::to_value(surface).unwrap()["root"].clone()
}

fn children(root: &Value) -> Vec<Value> {
    root["children"].as_array().cloned().unwrap_or_default()
}

fn stats_calls(host: &portaki_test_utils::MockHostFunctions) -> usize {
    host.connector_calls()
        .iter()
        .filter(|call| call.connector_id == "open-weather-statistics")
        .count()
}

#[test]
#[serial]
fn disabled_renders_an_empty_section_without_calling_anyone() {
    reset_test_harness();
    visitor(json!({})).run_with(|ctx, host| {
        assert!(ctx.is_public_visitor());
        let root = root(&render_property_public(ctx).unwrap());
        assert_eq!(root["type"], "Section");
        assert_eq!(root["title"], "i18n:public.title");
        assert_eq!(root["subtitle"], "i18n:public.eyebrow");
        assert!(children(&root).is_empty(), "{root}");
        assert!(host.connector_calls().is_empty());
    });
}

#[test]
#[serial]
fn without_todays_weather_the_section_is_empty() {
    reset_test_harness();
    with_season_cached(visitor(enabled()))
        .with_connector_error("open-weather", "current", "connector_egress_failed")
        .run(|ctx| {
            let root = root(&render_property_public(ctx).expect("never an error to the page"));
            assert!(children(&root).is_empty(), "{root}");
        });
}

#[test]
#[serial]
fn without_averages_only_todays_temperature_is_shown() {
    reset_test_harness();
    visitor(enabled())
        .with_connector_error("open-weather-statistics", "month", "connector_http_401")
        .run_with(|ctx, host| {
            let root = root(&render_property_public(ctx).expect("never an error to the page"));
            let text = root.to_string();
            assert!(text.contains(r#""type":"Temperature""#), "{text}");
            assert!(text.contains("i18n:weather.description.sunny"), "{text}");
            assert!(!text.contains(r#""type":"Grid""#), "{text}");
            // Au premier échec on n'insiste pas, et la position est notée pour un jour.
            assert_eq!(stats_calls(host), 1);
            assert!(host
                .kv_get("climate.unavailable.43.55.7.01")
                .unwrap()
                .is_some());
        });
}

#[test]
#[serial]
fn cached_averages_show_the_six_months_of_the_season_without_a_call() {
    reset_test_harness();
    with_season_cached(visitor(enabled())).run_with(|ctx, host| {
        let root = root(&render_property_public(ctx).unwrap());
        assert!(check_property_public_tree(&root).is_empty(), "{root}");
        let grid = children(&root)
            .into_iter()
            .find(|child| child["type"] == "Grid")
            .expect("a grid of months");
        let columns = grid["children"].as_array().unwrap();
        assert_eq!(columns.len(), 6);
        for (column, (_, key)) in columns.iter().zip(SEASON) {
            let text = column.to_string();
            assert!(text.contains(&format!("i18n:month.{key}")), "{text}");
            assert!(text.contains("19°"), "18.5 °C, arrondi : {text}");
        }
        assert_eq!(stats_calls(host), 0, "le KV suffit");
    });
}

#[test]
#[serial]
fn the_averages_fill_over_two_renders_within_the_call_budget() {
    reset_test_harness();
    let fetched = visitor(enabled())
        .with_connector_response("open-weather-statistics", "month", month_json())
        .run_with(|ctx, host| {
            let root = root(&render_property_public(ctx).unwrap());
            assert!(!root.to_string().contains(r#""type":"Grid""#), "{root}");
            let calls = host.connector_calls();
            assert_eq!(
                calls.len(),
                2 + 3,
                "5 appels par invocation, pas un de plus"
            );
            let args: Value = serde_json::from_str(
                &calls
                    .iter()
                    .find(|call| call.connector_id == "open-weather-statistics")
                    .unwrap()
                    .args_json,
            )
            .unwrap();
            assert_eq!(args, json!({ "lat": 43.55, "lon": 7.01, "month": 4 }));
            SEASON
                .iter()
                .filter_map(|(month, _)| {
                    host.kv_get(&month_key(*month))
                        .unwrap()
                        .map(|bytes| (month_key(*month), bytes))
                })
                .collect::<Vec<_>>()
        });
    assert_eq!(fetched.len(), 3);
    assert_eq!(fetched[0].1, b"22".to_vec(), "295.15 K = 22 °C");

    reset_test_harness();
    let mut second = visitor(enabled()).with_connector_response(
        "open-weather-statistics",
        "month",
        month_json(),
    );
    for (key, bytes) in fetched {
        second = second.with_kv(key, bytes);
    }
    second.run_with(|ctx, host| {
        let root = root(&render_property_public(ctx).unwrap());
        assert!(root.to_string().contains(r#""type":"Grid""#), "{root}");
        assert_eq!(stats_calls(host), 3);
    });
}

#[test]
#[serial]
fn monthly_averages_off_shows_today_only_and_asks_nothing() {
    reset_test_harness();
    with_season_cached(visitor(
        json!({ "public_enabled": true, "public_monthly_averages": false }),
    ))
    .run_with(|ctx, host| {
        let text = root(&render_property_public(ctx).unwrap()).to_string();
        assert!(text.contains(r#""type":"Temperature""#), "{text}");
        assert!(!text.contains(r#""type":"Grid""#), "{text}");
        assert_eq!(stats_calls(host), 0);
    });
}

/// La surface ne lit jamais le séjour : avec ou sans, le même arbre.
#[test]
#[serial]
fn the_stay_is_never_read() {
    reset_test_harness();
    let without = with_season_cached(visitor(enabled()))
        .run(|ctx| root(&render_property_public(ctx).unwrap()));
    reset_test_harness();
    let stay = portaki_sdk::context::StayContext {
        checkin_at: Some(Utc.with_ymd_and_hms(2026, 12, 20, 15, 0, 0).unwrap()),
        checkout_at: Some(Utc.with_ymd_and_hms(2026, 12, 27, 10, 0, 0).unwrap()),
        ..Default::default()
    };
    let with = with_season_cached(visitor(enabled()))
        .with_stay(stay)
        .run(|ctx| root(&render_property_public(ctx).unwrap()));
    assert_eq!(without, with);
}

#[test]
#[serial]
fn american_visitors_read_fahrenheit() {
    reset_test_harness();
    let text = with_season_cached(visitor(enabled())).run(|mut ctx| {
        ctx.locale = "en-US".into();
        root(&render_property_public(ctx).unwrap()).to_string()
    });
    assert!(text.contains("65°"), "18.5 °C = 65 °F : {text}");
}

/// Les mois courts existent dans les dix langues, et diffèrent d'une langue à l'autre.
#[test]
fn every_locale_names_the_twelve_months() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/i18n");
    let mut junes = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let bundle: Value =
            serde_json::from_str(&std::fs::read_to_string(entry.unwrap().path()).unwrap()).unwrap();
        for key in [
            "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
        ] {
            let label = bundle[format!("month.{key}")].as_str().unwrap_or_default();
            assert!(!label.trim().is_empty(), "month.{key}");
        }
        junes.push(bundle["month.jun"].as_str().unwrap().to_string());
    }
    assert_eq!(junes.len(), 10);
    assert!(junes.contains(&"Juin".to_string()) && junes.contains(&"Jun".to_string()));
}
