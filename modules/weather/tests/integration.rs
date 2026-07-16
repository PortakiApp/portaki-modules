//! Integration-style unit tests with `portaki-test-utils`.

use serial_test::serial;

use portaki_sdk::sdui::component::Component;
use portaki_sdk::sdui::surface::Surface;
use portaki_test_utils::{MockContext, Property};
use serde_json::json;
use std::sync::atomic::Ordering;

use weather::{
    get_current, get_forecast, on_booking_confirmed, refresh_forecast, render_explore_forecast,
    render_home_card, reset_test_harness, CONNECTOR_CURRENT_CALLS, CONNECTOR_FORECAST_CALLS,
};
use weather::{BookingConfirmedEvent, GetCurrentArgs, GetForecastArgs};

fn sample_current_json() -> String {
    json!({
        "main": { "temp": 21.5, "humidity": 55 },
        "weather": [{ "main": "Clear" }]
    })
    .to_string()
}

fn sample_forecast_json() -> String {
    let dates = [
        "2026-05-22",
        "2026-05-23",
        "2026-05-24",
        "2026-05-25",
        "2026-05-26",
    ];
    let conditions = ["Clear", "Clouds", "Rain", "Clouds", "Clear"];
    let list: Vec<serde_json::Value> = dates
        .iter()
        .zip(conditions.iter())
        .enumerate()
        .map(|(index, (date, condition))| {
            json!({
                "dt_txt": format!("{date} 12:00:00"),
                "main": {
                    "temp_min": 16.0 + index as f64,
                    "temp_max": 24.0 + index as f64
                },
                "weather": [{ "main": condition }]
            })
        })
        .collect();
    json!({ "list": list }).to_string()
}

fn contains_component_type(surface: &Surface, type_name: &str) -> bool {
    fn walk(node: &Component, type_name: &str) -> bool {
        let matches = match node {
            Component::Card(_) if type_name == "Card" => true,
            Component::Icon(_) if type_name == "Icon" => true,
            Component::Text(_) if type_name == "Text" => true,
            Component::EmptyState(_) if type_name == "EmptyState" => true,
            Component::Stack(_) if type_name == "Stack" => true,
            Component::Divider(_) if type_name == "Divider" => true,
            Component::InfoBanner(_) if type_name == "InfoBanner" => true,
            Component::Button(_) if type_name == "Button" => true,
            Component::Grid(_) if type_name == "Grid" => true,
            _ => false,
        };
        if matches {
            return true;
        }
        for child in child_components(node) {
            if walk(child, type_name) {
                return true;
            }
        }
        false
    }
    walk(&surface.root, type_name)
}

fn child_components(node: &Component) -> Vec<&Component> {
    match node {
        Component::Stack(inner) => inner.children.iter().collect(),
        Component::Divider(_) => Vec::new(),
        Component::Card(inner) => inner.children.iter().collect(),
        Component::Grid(inner) => inner.children.iter().collect(),
        Component::EmptyState(inner) => inner.children.iter().collect(),
        Component::Group(inner) => inner.children.iter().collect(),
        Component::Form(inner) => inner.children.iter().collect(),
        Component::Page(inner) => inner.children.iter().collect(),
        _ => Vec::new(),
    }
}

#[test]
#[serial]
fn home_card_renders_with_capability_pool() {
    reset_test_harness();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&["core.storage", "external.open-weather.pool"])
        .with_connector_response("open-weather", "current", sample_current_json())
        .with_connector_response("open-weather", "forecast", sample_forecast_json())
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "Card"));
            assert!(contains_component_type(&surface, "Stack"));
            assert!(contains_component_type(&surface, "Text"));
            assert!(contains_component_type(&surface, "Icon"));
            assert!(contains_component_type(&surface, "Grid"));
            assert!(contains_component_type(&surface, "Divider"));
        });
}

#[test]
#[serial]
fn home_card_renders_empty_state_without_capability() {
    reset_test_harness();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&["core.storage"])
        .run(|ctx| {
            let surface = render_home_card(ctx);
            assert!(contains_component_type(&surface, "EmptyState"));
        });
}

#[test]
#[serial]
fn get_current_uses_cache_on_second_call() {
    reset_test_harness();

    let builder = MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&["core.storage", "external.open-weather.pool"])
        .with_connector_response("open-weather", "current", sample_current_json())
        .with_connector_response("open-weather", "forecast", sample_forecast_json());

    builder.clone().run(|ctx| {
        get_current(
            ctx,
            GetCurrentArgs {
                lat: None,
                lng: None,
            },
        )
        .expect("first get_current");
    });

    let calls_after_first = CONNECTOR_CURRENT_CALLS.load(Ordering::SeqCst);
    assert_eq!(calls_after_first, 1);

    builder.run(|ctx| {
        get_current(
            ctx,
            GetCurrentArgs {
                lat: None,
                lng: None,
            },
        )
        .expect("cached get_current");
    });

    let calls_after_second = CONNECTOR_CURRENT_CALLS.load(Ordering::SeqCst);
    assert_eq!(calls_after_second, 1);
}

#[test]
#[serial]
fn refresh_forecast_invalidates_cache() {
    reset_test_harness();

    let builder = MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&["core.storage", "external.open-weather.pool"])
        .with_connector_response("open-weather", "current", sample_current_json())
        .with_connector_response("open-weather", "forecast", sample_forecast_json());

    builder.clone().run(|ctx| {
        get_current(
            ctx,
            GetCurrentArgs {
                lat: None,
                lng: None,
            },
        )
        .expect("warm cache");
    });

    builder.clone().run(|ctx| {
        refresh_forecast(ctx).expect("invalidate");
    });

    builder.run(|ctx| {
        get_current(
            ctx,
            GetCurrentArgs {
                lat: None,
                lng: None,
            },
        )
        .expect("refresh after invalidate");
    });

    assert_eq!(CONNECTOR_CURRENT_CALLS.load(Ordering::SeqCst), 2);
}

#[test]
#[serial]
fn forecast_renders_5_days() {
    reset_test_harness();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&["core.storage", "external.open-weather.byok"])
        .with_connector_response("open-weather", "current", sample_current_json())
        .with_connector_response("open-weather", "forecast", sample_forecast_json())
        .run(|ctx| {
            let surface = render_explore_forecast(ctx);
            assert!(contains_component_type(&surface, "Grid"));
            assert!(contains_component_type(&surface, "InfoBanner"));
            assert!(contains_component_type(&surface, "Button"));
            let json = serde_json::to_string(&surface).expect("surface json");
            // now icon + 5 day icons
            assert!(json.matches("\"Icon\"").count() >= 6);
        });
}

#[test]
#[serial]
fn on_booking_confirmed_prewarms_cache() {
    reset_test_harness();

    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&["core.storage", "external.open-weather.pool"])
        .with_connector_response("open-weather", "current", sample_current_json())
        .with_connector_response("open-weather", "forecast", sample_forecast_json())
        .run(|ctx| {
            on_booking_confirmed(
                ctx,
                BookingConfirmedEvent {
                    id: uuid::Uuid::new_v4(),
                    property_id: Property::default().id,
                },
            )
            .expect("prewarm");
        });

    assert!(CONNECTOR_CURRENT_CALLS.load(Ordering::SeqCst) >= 1);
    assert!(CONNECTOR_FORECAST_CALLS.load(Ordering::SeqCst) >= 1);
}

#[test]
#[serial]
fn get_forecast_returns_five_days() {
    reset_test_harness();

    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&["core.storage", "external.open-weather.pool"])
        .with_connector_response("open-weather", "current", sample_current_json())
        .with_connector_response("open-weather", "forecast", sample_forecast_json())
        .run(|ctx| {
            let forecast = get_forecast(
                ctx,
                GetForecastArgs {
                    lat: None,
                    lng: None,
                    days: Some(5),
                },
            )
            .expect("forecast");
            assert_eq!(forecast.days.len(), 5);
        });
}

#[test]
#[serial]
#[ignore = "requires wasm32 build pipeline — run on main CI"]
fn wasm_render_home_card_end_to_end() {
    // Placeholder for CI wasm snapshot test (portaki build + wasmtime harness).
}
