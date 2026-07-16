//! WeatherCache persistence via `host::repo` (gateway-backed in Wasm).

use chrono::{DateTime, Duration, Utc};
use portaki_sdk::host::repo::{self, eq, find, Query};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use uuid::Uuid;

use crate::entities::{WeatherCache, WeatherUnits};
use crate::weather::{
    CachedCurrentPayload, CachedForecastPayload, ForecastDayView, WeatherCurrent, WeatherForecast,
    CURRENT_CACHE_TTL_SECS, FORECAST_CACHE_TTL_SECS,
};

use std::cell::RefCell;

thread_local! {
    static TEST_CACHE_ROWS: RefCell<Vec<WeatherCache>> = const { RefCell::new(Vec::new()) };
}

/// Clears the in-memory cache used by unit tests.
#[allow(dead_code)]
pub fn reset_test_cache() {
    TEST_CACHE_ROWS.with(|rows| rows.borrow_mut().clear());
}

/// Resets in-memory cache and connector call counters between tests.
#[allow(dead_code)]
pub fn reset_test_harness() {
    reset_test_cache();
    crate::weather::CONNECTOR_CURRENT_CALLS.store(0, std::sync::atomic::Ordering::SeqCst);
    crate::weather::CONNECTOR_FORECAST_CALLS.store(0, std::sync::atomic::Ordering::SeqCst);
}

fn test_store_rows() -> Vec<WeatherCache> {
    TEST_CACHE_ROWS.with(|rows| rows.borrow().clone())
}

fn test_store_upsert(row: WeatherCache) {
    TEST_CACHE_ROWS.with(|rows| {
        let mut guard = rows.borrow_mut();
        if let Some(index) = guard.iter().position(|item| item.id == row.id) {
            guard[index] = row;
        } else if let Some(index) = guard.iter().position(|item| {
            (item.lat - row.lat).abs() < f64::EPSILON && (item.lng - row.lng).abs() < f64::EPSILON
        }) {
            guard[index] = row;
        } else {
            guard.push(row);
        }
    });
}

/// Finds a non-expired cache row for coordinates.
pub fn find_valid(lat: f64, lng: f64, now: DateTime<Utc>) -> Result<Option<WeatherCache>> {
    let rows = load_rows(lat, lng)?;
    Ok(rows.into_iter().find(|row| row.expires_at > now))
}

/// Stores or updates cache for coordinates.
pub fn upsert(
    lat: f64,
    lng: f64,
    units: WeatherUnits,
    current: &CachedCurrentPayload,
    forecast: &CachedForecastPayload,
    current_expires: DateTime<Utc>,
    forecast_expires: DateTime<Utc>,
) -> Result<()> {
    let now = time::now()?;
    let expires_at = current_expires.max(forecast_expires);
    let current_json = serde_json::to_string(current).map_err(map_serde_error)?;
    let forecast_json = serde_json::to_string(forecast).map_err(map_serde_error)?;

    let existing = load_rows(lat, lng)?;
    if let Some(mut row) = existing.into_iter().next() {
        row.current_json = current_json;
        row.forecast_json = forecast_json;
        row.units = units;
        row.fetched_at = now;
        row.expires_at = expires_at;
        row.updated_at = now;
        persist_row(row)?;
        return Ok(());
    }

    let row = WeatherCache {
        id: Uuid::new_v4(),
        lat,
        lng,
        current_json,
        forecast_json,
        units,
        fetched_at: now,
        expires_at,
        created_at: now,
        updated_at: now,
    };
    persist_row(row)
}

/// Deletes cache rows for coordinates.
pub fn invalidate(lat: f64, lng: f64) -> Result<()> {
    let rows = load_rows(lat, lng)?;
    for row in rows {
        delete_row(row.id)?;
    }
    Ok(())
}

/// Reads cached current weather if still valid.
pub fn read_current(
    lat: f64,
    lng: f64,
    units: WeatherUnits,
    now: DateTime<Utc>,
) -> Result<Option<WeatherCurrent>> {
    let Some(row) = find_valid(lat, lng, now)? else {
        return Ok(None);
    };
    let payload: CachedCurrentPayload =
        serde_json::from_str(&row.current_json).map_err(map_serde_error)?;
    let condition = payload.condition.clone();
    Ok(Some(WeatherCurrent {
        temp_c: payload.temp_c,
        condition: condition.clone(),
        humidity: payload.humidity,
        uv_index: payload.uv_index,
        wind_speed_ms: payload.wind_speed_ms,
        description_key: crate::weather::description_key_for_condition(&condition),
        units,
        fetched_at: row.fetched_at,
    }))
}

/// Reads cached forecast if still valid.
pub fn read_forecast(
    lat: f64,
    lng: f64,
    units: WeatherUnits,
    now: DateTime<Utc>,
) -> Result<Option<WeatherForecast>> {
    let Some(row) = find_valid(lat, lng, now)? else {
        return Ok(None);
    };
    let payload: CachedForecastPayload =
        serde_json::from_str(&row.forecast_json).map_err(map_serde_error)?;
    Ok(Some(WeatherForecast {
        days: payload.days,
        units,
        fetched_at: row.fetched_at,
    }))
}

/// Writes current payload with a 1h TTL boundary.
pub fn store_current(
    lat: f64,
    lng: f64,
    units: WeatherUnits,
    current: &WeatherCurrent,
    forecast_days: &[ForecastDayView],
) -> Result<()> {
    let now = time::now()?;
    let current_expires = now + Duration::seconds(CURRENT_CACHE_TTL_SECS);
    let forecast_expires = now + Duration::seconds(FORECAST_CACHE_TTL_SECS);
    let current_payload = CachedCurrentPayload {
        temp_c: current.temp_c,
        condition: current.condition.clone(),
        humidity: current.humidity,
        uv_index: current.uv_index,
        wind_speed_ms: current.wind_speed_ms,
    };
    let forecast_payload = CachedForecastPayload {
        days: forecast_days.to_vec(),
    };
    upsert(
        lat,
        lng,
        units,
        &current_payload,
        &forecast_payload,
        current_expires,
        forecast_expires,
    )
}

fn load_rows(lat: f64, lng: f64) -> Result<Vec<WeatherCache>> {
    if in_memory_cache_enabled() {
        let rows: Vec<WeatherCache> = test_store_rows()
            .into_iter()
            .filter(|row| {
                (row.lat - lat).abs() < f64::EPSILON && (row.lng - lng).abs() < f64::EPSILON
            })
            .collect();
        if !rows.is_empty() {
            return Ok(rows);
        }
    }

    let query = Query::<WeatherCache>::new()
        .r#where(eq("lat", lat))
        .r#where(eq("lng", lng))
        .limit(10);
    let page = find::<WeatherCache, WeatherCache>(query)?;
    Ok(page.items)
}

fn persist_row(row: WeatherCache) -> Result<()> {
    if in_memory_cache_enabled() {
        test_store_upsert(row);
        return Ok(());
    }
    let _ = repo::create::<WeatherCache, WeatherCache, WeatherCache>(row)?;
    Ok(())
}

fn delete_row(id: Uuid) -> Result<()> {
    if in_memory_cache_enabled() {
        TEST_CACHE_ROWS.with(|rows| {
            rows.borrow_mut().retain(|row| row.id != id);
        });
        return Ok(());
    }
    repo::delete::<WeatherCache>(id)?;
    Ok(())
}

fn map_serde_error(error: serde_json::Error) -> PortakiError {
    PortakiError::Storage(format!("cache JSON error: {error}"))
}

/// Uses thread-local cache for unit/integration tests and `portaki dev` (no gateway repo).
fn in_memory_cache_enabled() -> bool {
    cfg!(test) || cfg!(debug_assertions)
}
