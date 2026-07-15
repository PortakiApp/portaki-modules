<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://portaki.app/logo-dark.svg">
    <img src="https://portaki.app/logo-light.svg" width="177" height="48" alt="Portaki">
  </picture>
</p>

<h1 align="center">portaki-modules</h1>

<p align="center">
  <strong>Official Portaki Wasm guest modules monorepo</strong><br>
  Independently versioned Extism modules, published to GitHub Container Registry as public OCI images.
</p>

<p align="center">
  <a href="https://github.com/PortakiApp/portaki-modules/actions/workflows/ci.yml"><img src="https://github.com/PortakiApp/portaki-modules/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue.svg" alt="License Apache-2.0"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.75+-dea584?logo=rust&logoColor=white" alt="Rust 1.75+"></a>
  <a href="https://extism.org/"><img src="https://img.shields.io/badge/Extism-Wasm-7C3AED" alt="Extism"></a>
  <a href="https://github.com/orgs/PortakiApp/packages?repo_name=portaki-modules"><img src="https://img.shields.io/badge/GHCR-portaki--modules-*-2496ED?logo=github" alt="GHCR"></a>
  <a href="https://portaki.app"><img src="https://img.shields.io/badge/site-portaki.app-f59e0b" alt="portaki.app"></a>
</p>

<p align="center">
  <a href="#modules">Modules</a> ·
  <a href="#requirements">Requirements</a> ·
  <a href="#development">Development</a> ·
  <a href="#publishing">Publishing</a> ·
  <a href="CONTRIBUTING.md">Contributing</a> ·
  <a href="SECURITY.md">Security</a>
</p>

---

Each crate under `modules/` is a Portaki guest module. Authoring uses [`portaki-sdk`](https://github.com/PortakiApp/portaki-sdk) (`portaki` CLI).

On every push to **`main`**, CI builds and publishes:

`ghcr.io/portakiapp/portaki-modules-<module-id>:<semver>`

## Why this monorepo?

- **One repo per ecosystem** — shared CI, shared SDK pins, consistent lint/build gates
- **Independent versions** — each module bumps its own `Cargo.toml` semver
- **OCI-first** — dash-named public GHCR packages (`portaki-modules-weather`, …)
- **Pattern A** — official modules live here; third-party modules use standalone repos + the same SDK

## Modules

| Module | OCI image | Description |
|--------|-----------|-------------|
| [`weather`](./modules/weather) | `ghcr.io/portakiapp/portaki-modules-weather:<semver>` | Current weather and 5-day forecast |

## Structure

```
portaki-modules/
├── Cargo.toml                 # workspace + shared SDK git deps
├── modules/
│   └── weather/
└── .github/workflows/
    └── ci.yml                 # quality gates; publish to GHCR on main
```

## Requirements

- Rust **1.75+**
- Target `wasm32-unknown-unknown`
- [`portaki` CLI](https://github.com/PortakiApp/portaki-sdk) from `portaki-sdk`

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

See [CONTRIBUTING.md](./CONTRIBUTING.md) for adding a new module.

## Publishing

1. Bump `version` in `modules/<id>/Cargo.toml`
2. Merge to **`main`**
3. CI publishes `ghcr.io/portakiapp/portaki-modules-<id>:<semver>`

GHCR packages are **public**. CI publishes with `GITHUB_TOKEN` (`packages: write`). Local publish needs a classic PAT with `write:packages` (or `docker login ghcr.io`).

## Related repositories

| Repository | Role |
|------------|------|
| [portaki-sdk](https://github.com/PortakiApp/portaki-sdk) | Rust SDK + `portaki` CLI |
| [portaki.app](https://portaki.app) | Product site |

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) and the [Code of Conduct](./CODE_OF_CONDUCT.md).

Security issues: [SECURITY.md](./SECURITY.md) — do not open a public issue.

## License

[Apache-2.0](./LICENSE) · Copyright 2026 Syntax Labs
