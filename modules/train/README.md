# train

Official Portaki train module — nearest station schedule glance for guest booklets.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`train`

OCI image: `ghcr.io/portakiapp/portaki-modules-train:<semver>`

## Capabilities

None. v0.1 has no host editor and no storage — station info and destination
schedules are static Rust constants in `src/content.rs`. A future pass will
read this from module config and/or a Navitia connector.

## Content model

Static, hardcoded in `src/content.rs`:

- Nearest station label + distance (`DEFAULT_STATION_LABEL`, `default_station_distance`)
- Destinations: `Nice-Ville`, `Cannes`, `Monaco`, `Grasse`
- Mock TER SUD PACA departure times per destination (`schedule_for`)

## Surfaces

| Shell | Surface id | Description |
|-------|------------|--------------|
| guest | `home.card` | Mixed-destination departure board glance (4 rows) |
| guest | `explore.detail` | From/to header, destination filter chips, next departures |

Guest route: `pathSegment = "train"` (see `portaki.module.json`).

Destination filter chips re-navigate to `train` with `{ "dest": "<destination>" }`
params, read back via `ctx.input.dest` in `render_explore_detail`.

## Development

```bash
cargo test -p train
cd modules/train
portaki build --release
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
