-- weather @ schema v1 — idempotent (module_weather)

CREATE SCHEMA IF NOT EXISTS module_weather;

CREATE TABLE IF NOT EXISTS module_weather.weather_cache (
    id UUID PRIMARY KEY,
    property_id UUID NOT NULL,
    lat DOUBLE PRECISION NOT NULL,
    lng DOUBLE PRECISION NOT NULL,
    current_json TEXT NOT NULL DEFAULT '',
    forecast_json TEXT NOT NULL DEFAULT '',
    units TEXT NOT NULL DEFAULT 'celsius',
    fetched_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS weather_cache_property_lat_lng_idx
    ON module_weather.weather_cache (property_id, lat, lng);
