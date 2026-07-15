# weather

Portaki official weather module — current conditions and 5-day forecast for guest booklets and the host dashboard.

Part of [portaki-modules-rust](https://github.com/PortakiApp/portaki-modules-rust) monorepo (migrated from standalone `portaki-module-weather`).

## Module id

`weather` — OCI image `ghcr.io/portakiapp/portaki-modules-weather:<semver>`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | KV config + `WeatherCache` entity |
| `external.open-weather.pool` | No | Platform OpenWeather pool token |
| `external.open-weather.byok` | No | Property BYOK OpenWeather key |

Without pool or BYOK, guest surfaces render an `EmptyState` with upgrade/BYOK guidance.

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Current temperature + condition + optional UV badge |
| guest | `explore.forecast` | 5-day forecast grid |
| host | `main` | Units and refresh interval form |

## Queries & commands

- `getCurrent` — cache TTL 1h
- `getForecast` — cache TTL 6h (5 days)
- `refreshForecast` — invalidates cache for property coordinates
- `updateConfig` — persists host settings in KV
- Event `core.booking.confirmed` — pre-warms cache

## Development

From monorepo root:

```bash
cargo test -p weather
cd modules/weather
portaki build --release
portaki lint
```

## Publishing

Bump `version` in `Cargo.toml`, merge to `main` — CI publishes `ghcr.io/portakiapp/portaki-modules/weather:<semver>`. See root README.

## License

Apache-2.0
