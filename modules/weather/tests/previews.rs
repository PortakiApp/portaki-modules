//! Aperçus du catalogue public — voir `support/previews.rs`.

#[path = "../../../support/previews.rs"]
mod previews;

use portaki_sdk::capability;
use serde_json::{json, Value};
use weather::{render_explore_forecast, render_upcoming_card};

/// La météo vient d'OpenWeather : l'aperçu lui substitue une réponse d'exemple, une semaine de
/// début juin ensoleillée avec un passage pluvieux, au format que le connecteur renvoie.
fn sample_current() -> String {
    json!({
        "name": "Cannes",
        "main": { "temp": 23.4, "humidity": 58 },
        "wind": { "speed": 3.2 },
        "weather": [{ "main": "Clear" }]
    })
    .to_string()
}

fn sample_forecast() -> String {
    let days = [
        ("2026-06-01", "Clear", 17.0, 25.0, 0.0, 55, 3.1),
        ("2026-06-02", "Clouds", 17.5, 24.0, 0.2, 62, 4.0),
        ("2026-06-03", "Rain", 16.0, 21.0, 0.8, 81, 6.5),
        ("2026-06-04", "Clouds", 16.5, 23.0, 0.3, 66, 4.4),
        ("2026-06-05", "Clear", 18.0, 27.0, 0.0, 52, 2.8),
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
fn previews_match_the_rendered_surfaces() {
    let root = env!("CARGO_MANIFEST_DIR");
    let context = previews::guest(root)
        .with_extra_capability_ids(&[capability::external::OPEN_WEATHER_POOL.as_str()])
        .with_connector_response("open-weather", "current", sample_current())
        .with_connector_response("open-weather", "forecast", sample_forecast());
    let forecast = context.clone().run(render_explore_forecast);
    let upcoming = context.run(render_upcoming_card);
    previews::check(
        root,
        concat!(env!("OUT_DIR"), "/portaki-emissions"),
        vec![("explore.forecast", forecast), ("upcoming.card", upcoming)],
    );
}
