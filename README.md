# portaki-modules-rust

Official [Portaki](https://github.com/PortakiApp) module monorepo (Pattern A). Each crate under `modules/` is an independently versioned Wasm module published to **GitHub Container Registry** (GHCR).

## Structure

```
portaki-modules-rust/
├── Cargo.toml              # workspace + shared SDK git deps
├── modules/
│   └── weather/            # weather module (migrated from portaki-module-weather)
└── .github/workflows/
    └── ci.yml              # fmt, clippy, test, lint/build; publish to GHCR on main
```

## Modules

| Module | OCI image | Description |
|--------|-----------|-------------|
| `weather` | `ghcr.io/portakiapp/portaki-modules-weather:<semver>` | Current weather + 5-day forecast |

## Development

```bash
rustup target add wasm32-unknown-unknown

cargo install --git https://github.com/PortakiApp/portaki-sdk-rust --branch main --locked portaki-cli

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

cd modules/weather
portaki build --release
portaki lint
```

## Publishing

Push to **`main`** with an updated `version` in `modules/<id>/Cargo.toml`. CI runs quality gates, then publishes every module to `ghcr.io/portakiapp/portaki-modules-<id>:<semver>`.

No git tags for now — **release-please** later.

## Migration note

The `weather` module was migrated from standalone [portaki-module-weather](https://github.com/PortakiApp/portaki-module-weather) (C1 pilot). Standalone repo v0.1.0 was never published.

See [CONTRIBUTING.md](./CONTRIBUTING.md) for adding new modules.

## License

Apache-2.0 — see [LICENSE](./LICENSE).
