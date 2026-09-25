//! Integration-style unit tests with `portaki-test-utils`.

use portaki_sdk::capability;
use serial_test::serial;

use portaki_test_utils::{MockContext, Property, SurfaceAssertions};
use serde_json::json;

#[path = "../../../support/config_form.rs"]
mod config_form;
use std::sync::atomic::Ordering;

use portaki_sdk::prelude::EmailTemplateKey;
use weather::{
    email_context, get_current, get_forecast, on_booking_confirmed, refresh_forecast,
    render_explore_forecast, render_home_card, render_upcoming_card, reset_test_harness,
    CONNECTOR_CURRENT_CALLS, CONNECTOR_FORECAST_CALLS,
};
use weather::{BookingConfirmedEvent, EmailContextArgs, GetCurrentArgs, GetForecastArgs};

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

#[test]
#[serial]
fn home_card_renders_with_capability_pool() {
    reset_test_harness();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::external::OPEN_WEATHER_POOL,
        ])
        .with_connector_response("open-weather", "current", sample_current_json())
        .with_connector_response("open-weather", "forecast", sample_forecast_json())
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("render");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Stack"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Text"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Icon"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Grid"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Divider"));
        });
}

#[test]
#[serial]
fn upcoming_card_renders_compact_headline() {
    reset_test_harness();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::external::OPEN_WEATHER_POOL,
        ])
        .with_connector_response("open-weather", "current", sample_current_json())
        .with_connector_response("open-weather", "forecast", sample_forecast_json())
        .run(|ctx| {
            let surface = render_upcoming_card(ctx).expect("render");
            assert!(SurfaceAssertions::new(&surface).contains_type("Card"));
            assert!(SurfaceAssertions::new(&surface).contains_type("Text"));
            // Compact card must not embed the full forecast strip.
            assert!(!SurfaceAssertions::new(&surface).contains_type("Grid"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("upcoming.card"));
        });
}

#[test]
#[serial]
fn upcoming_card_renders_empty_state_without_capability() {
    reset_test_harness();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let surface = render_upcoming_card(ctx).expect("render");
            assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
        });
}

#[test]
#[serial]
fn email_context_returns_french_summary() {
    reset_test_harness();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::external::OPEN_WEATHER_POOL,
        ])
        .with_connector_response("open-weather", "current", sample_current_json())
        .with_connector_response("open-weather", "forecast", sample_forecast_json())
        .with_translation("email.place.inCity", "à {name}")
        .with_translation("email.place.onSite", "sur place")
        .with_translation(
            "email.weather.summary",
            "Météo {place} aujourd'hui : {emoji} {temp}°C, {condition}.",
        )
        .with_translation("email.condition.sunny", "ciel dégagé")
        .with_translation("email.condition.variable", "conditions variables")
        .run(|ctx| {
            let response = email_context(
                ctx,
                EmailContextArgs {
                    template_key: Some(EmailTemplateKey::ArrivalDay),
                    address_hint: Some("Cap d'Antibes, France".into()),
                    locale: Some("fr".into()),
                },
            )
            .expect("emailContext");
            let summary = response.weather_summary.expect("summary");
            assert!(summary.contains("aujourd'hui"));
            assert!(summary.contains("°C"));
        });
}

#[test]
#[serial]
fn home_card_renders_empty_state_without_capability() {
    reset_test_harness();
    MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[capability::core::STORAGE])
        .run(|ctx| {
            let surface = render_home_card(ctx).expect("render");
            assert!(SurfaceAssertions::new(&surface).contains_type("EmptyState"));
        });
}

#[test]
#[serial]
fn get_current_uses_cache_on_second_call() {
    reset_test_harness();

    let builder = MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::external::OPEN_WEATHER_POOL,
        ])
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
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::external::OPEN_WEATHER_POOL,
        ])
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
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::external::OPEN_WEATHER_BYOK,
        ])
        .with_connector_response("open-weather", "current", sample_current_json())
        .with_connector_response("open-weather", "forecast", sample_forecast_json())
        .run(|ctx| {
            let surface = render_explore_forecast(ctx).expect("render");
            assert!(SurfaceAssertions::new(&surface).contains_type("Grid"));
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains("explore.forecast.hint"));
            assert!(!json.contains("sheet.assistant.tip"));
            assert!(!json.contains("sheet.contactHost"));
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
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::external::OPEN_WEATHER_POOL,
        ])
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
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::external::OPEN_WEATHER_POOL,
        ])
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

#[test]
#[serial]
fn the_host_form_sends_the_declared_keys() {
    MockContext::host()
        .with_capabilities(&[capability::core::STORAGE])
        .with_config(&json!({ "units": "fahrenheit" }))
        .run(|ctx| {
            let config = weather::ModuleConfig::load(&ctx).expect("config");
            assert_eq!(config.units, weather::WeatherUnits::Fahrenheit);
            assert_eq!(config.refresh_interval, "1h");
            let surface = weather::render_host_main(ctx).expect("host main");
            config_form::assert_form_matches_config(
                concat!(env!("OUT_DIR"), "/portaki-emissions"),
                &surface,
                &[],
            );
            let json = serde_json::to_string(&surface).expect("surface json");
            assert!(json.contains(r#""value":"fahrenheit""#), "{json}");
        });
}

/// Not geocoded: every consumer stays quiet — a guest empty state, no email sentence, no
/// prewarm, no invalidation, a query error — and not one network call. Never a default position.
#[test]
#[serial]
fn without_coordinates_there_is_no_weather_and_no_call() {
    reset_test_harness();
    let builder = MockContext::guest()
        .with_property(Property::default())
        .with_capabilities(&[
            capability::core::STORAGE,
            capability::external::OPEN_WEATHER_POOL,
        ])
        .with_coordinates(None)
        .with_connector_response("open-weather", "current", sample_current_json())
        .with_connector_response("open-weather", "forecast", sample_forecast_json());

    builder.clone().run(|ctx| {
        for surface in [
            render_home_card(ctx.clone()),
            render_upcoming_card(ctx.clone()),
            render_explore_forecast(ctx.clone()),
        ] {
            let json = serde_json::to_string(&surface.expect("render")).expect("json");
            assert!(json.contains("EmptyState"), "{json}");
            assert!(json.contains("guest.noLocation.description"), "{json}");
        }
        let email = email_context(
            ctx.clone(),
            EmailContextArgs {
                template_key: Some(EmailTemplateKey::ArrivalDay),
                ..EmailContextArgs::default()
            },
        )
        .expect("emailContext");
        assert_eq!(email.weather_summary, None);
        refresh_forecast(ctx.clone()).expect("refreshForecast");
        let error = get_current(
            ctx,
            GetCurrentArgs {
                lat: None,
                lng: None,
            },
        )
        .expect_err("getCurrent");
        assert!(
            error.to_string().contains("property_not_geocoded"),
            "{error}"
        );
    });
    builder.run(|ctx| {
        on_booking_confirmed(
            ctx,
            BookingConfirmedEvent {
                id: uuid::Uuid::new_v4(),
                property_id: Property::default().id,
            },
        )
        .expect("booking confirmed");
    });

    assert_eq!(CONNECTOR_CURRENT_CALLS.load(Ordering::SeqCst), 0);
    assert_eq!(CONNECTOR_FORECAST_CALLS.load(Ordering::SeqCst), 0);
}
