//! Les sept cas pathologiques de la sandbox — voir `support/scenarios.rs`.

#[path = "../../../support/scenarios.rs"]
mod scenarios;

use portaki_sdk::capability;
use portaki_test_utils::MockContextBuilder;
use serde_json::{json, Value};
use serial_test::serial;
use weather::reset_test_harness;

/// OpenWeather branché (pool Portaki) : conditions du jour et cinq jours à partir de l'horloge
/// des cas, au format que le connecteur renvoie.
fn setup(builder: MockContextBuilder) -> MockContextBuilder {
    builder
        .with_extra_capability_ids(&[capability::external::OPEN_WEATHER_POOL.as_str()])
        .with_connector_response("open-weather", "current", current())
        .with_connector_response("open-weather", "forecast", forecast())
}

fn current() -> String {
    json!({
        "name": "Cannes",
        "main": { "temp": 23.4, "humidity": 58 },
        "wind": { "speed": 3.2 },
        "weather": [{ "main": "Clear" }]
    })
    .to_string()
}

fn forecast() -> String {
    let days = [
        ("2026-06-15", "Clear", 17.0, 25.0, 0.0, 55, 3.1),
        ("2026-06-16", "Clouds", 17.5, 24.0, 0.2, 62, 4.0),
        ("2026-06-17", "Rain", 16.0, 21.0, 0.8, 81, 6.5),
        ("2026-06-18", "Clouds", 16.5, 23.0, 0.3, 66, 4.4),
        ("2026-06-19", "Clear", 18.0, 27.0, 0.0, 52, 2.8),
    ];
    let list: Vec<Value> = days
        .iter()
        .map(|(date, condition, min, max, pop, humidity, wind)| {
            json!({
                "dt_txt": format!("{date} 12:00:00"),
                "main": { "temp_min": min, "temp_max": max, "humidity": humidity },
                "pop": pop,
                "wind": { "speed": wind },
                "weather": [{ "main": condition }]
            })
        })
        .collect();
    json!({ "city": { "name": "Cannes" }, "list": list }).to_string()
}

#[test]
#[serial]
fn every_surface_holds_on_every_case() {
    reset_test_harness();
    scenarios::check_surfaces(env!("CARGO_MANIFEST_DIR"), setup);
}

#[test]
#[serial]
fn every_example_runs() {
    reset_test_harness();
    scenarios::check_examples(concat!(env!("OUT_DIR"), "/portaki-emissions"), setup, &[]);
}
