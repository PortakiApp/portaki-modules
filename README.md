# portaki-modules

[![CI](https://github.com/PortakiApp/portaki-modules/actions/workflows/ci.yml/badge.svg)](https://github.com/PortakiApp/portaki-modules/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](./LICENSE)

Official [Portaki](https://portaki.app) Wasm modules monorepo.

Each crate under `modules/` is an independently versioned guest module. On every push to **`main`**, CI builds and publishes public OCI images to GitHub Container Registry (GHCR):

`ghcr.io/portakiapp/portaki-modules-<module-id>:<semver>`

Module authoring uses the Rust SDK in [`portaki-sdk`](https://github.com/PortakiApp/portaki-sdk) (`portaki` CLI).

## Structure

```
portaki-modules/
├── Cargo.toml                 # workspace + shared SDK git deps
├── modules/
│   └── weather/               # current conditions + 5-day forecast
└── .github/workflows/
    └── ci.yml                 # quality gates; publish to GHCR on main
```

## Modules

| Module | OCI image | Description |
|--------|-----------|-------------|
| [`weather`](./modules/weather) | `ghcr.io/portakiapp/portaki-modules-weather:<semver>` | Current weather and 5-day forecast |

## Requirements

- Rust **1.75+**
- `wasm32-unknown-unknown` target
- [`portaki` CLI](https://github.com/PortakiApp/portaki-sdk) from the SDK repo

```bash
rustup target add wasm32-unknown-unknown
cargo install --git https://github.com/PortakiApp/portaki-sdk --branch main --locked portaki-cli
```

## Development

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

cd modules/weather
portaki build --release
portaki lint
```

## Publishing

1. Bump `version` in `modules/<id>/Cargo.toml`.
2. Merge to **`main`**.
3. CI publishes `ghcr.io/portakiapp/portaki-modules-<id>:<semver>`.

Package visibility is **public** so the Portaki module runtime can discover and pull without private package grants. Writes still require CI `packages: write` (or a classic PAT with `write:packages` for local publish).

## Related repositories

| Repository | Role |
|------------|------|
| [portaki-sdk](https://github.com/PortakiApp/portaki-sdk) | Rust SDK + `portaki` CLI |
| [portaki-platform](https://github.com/PortakiApp/portaki-platform) | Orchestrator + module runtime |

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md). Security reports: [SECURITY.md](./SECURITY.md).

## License

Apache-2.0 — see [LICENSE](./LICENSE).

Copyright 2026 Syntax Labs.
