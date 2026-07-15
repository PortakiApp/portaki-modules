# weather

Official Portaki weather module — current conditions and a 5-day forecast for guest booklets and the host dashboard.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`weather`

OCI image: `ghcr.io/portakiapp/portaki-modules-weather:<semver>`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | KV config + `WeatherCache` entity |
| `external.open-weather.pool` | No | Platform OpenWeather pool token |
| `external.open-weather.byok` | No | Property BYOK OpenWeather key |

Without pool or BYOK access, guest surfaces render an empty state with upgrade / BYOK guidance.

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Current temperature, condition, optional UV badge |
| guest | `explore.forecast` | 5-day forecast grid |
| host | `main` | Units and refresh interval form |

## Queries and commands

- `getCurrent` — cache TTL 1h
- `getForecast` — cache TTL 6h (5 days)
- `refreshForecast` — invalidates cache for property coordinates
- `updateConfig` — persists host settings in KV
- Event `core.booking.confirmed` — pre-warms the cache

## Development

From the monorepo root:

```bash
cargo test -p weather
cd modules/weather
portaki build --release
portaki lint
```

## Publishing

Bump `version` in `Cargo.toml`, merge to `main`. CI publishes `ghcr.io/portakiapp/portaki-modules-weather:<semver>`. See the [root README](../../README.md).

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
