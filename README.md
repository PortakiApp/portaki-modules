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
| [`access-guide`](./modules/access-guide) | `ghcr.io/portakiapp/portaki-modules-access-guide:<semver>` | Arrival steps, codes, and parking |
| [`appliances`](./modules/appliances) | `ghcr.io/portakiapp/portaki-modules-appliances:<semver>` | Device guides and safety notice |
| [`checklist`](./modules/checklist) | `ghcr.io/portakiapp/portaki-modules-checklist:<semver>` | Checkout checklist with guest toggles |
| [`emergency-contacts`](./modules/emergency-contacts) | `ghcr.io/portakiapp/portaki-modules-emergency-contacts:<semver>` | Useful numbers and host line |
| [`ev-parking`](./modules/ev-parking) | `ghcr.io/portakiapp/portaki-modules-ev-parking:<semver>` | EV spot, gate code, charger PIN with timed reveal |
| [`events`](./modules/events) | `ghcr.io/portakiapp/portaki-modules-events:<semver>` | Host-curated local events and map |
| [`facility-hours`](./modules/facility-hours) | `ghcr.io/portakiapp/portaki-modules-facility-hours:<semver>` | Pool, spa, and shared amenity schedules |
| [`guest-reviews`](./modules/guest-reviews) | `ghcr.io/portakiapp/portaki-modules-guest-reviews:<semver>` | Post-stay thank-you and review CTAs |
| [`ical-sync`](./modules/ical-sync) | `ghcr.io/portakiapp/portaki-modules-ical-sync:<semver>` | Host iCal / Airbnb calendar feed import |
| [`issue-report`](./modules/issue-report) | `ghcr.io/portakiapp/portaki-modules-issue-report:<semver>` | In-stay problem reports for the host |
| [`local-guide`](./modules/local-guide) | `ghcr.io/portakiapp/portaki-modules-local-guide:<semver>` | Nearby spots and host picks |
| [`lost-found`](./modules/lost-found) | `ghcr.io/portakiapp/portaki-modules-lost-found:<semver>` | Guest lost / found item reports |
| [`nuki`](./modules/nuki) | `ghcr.io/portakiapp/portaki-modules-nuki:<semver>` | Nuki smart-lock provider for access-guide |
| [`pre-arrival-form`](./modules/pre-arrival-form) | `ghcr.io/portakiapp/portaki-modules-pre-arrival-form:<semver>` | ETA, occasion, allergies, message to host |
| [`rules`](./modules/rules) | `ghcr.io/portakiapp/portaki-modules-rules:<semver>` | Structured bilingual house rules |
| [`sections`](./modules/sections) | `ghcr.io/portakiapp/portaki-modules-sections:<semver>` | Editorial title + markdown body blocks |
| [`train`](./modules/train) | `ghcr.io/portakiapp/portaki-modules-train:<semver>` | Nearby station departure board |
| [`waste-recycling`](./modules/waste-recycling) | `ghcr.io/portakiapp/portaki-modules-waste-recycling:<semver>` | Bins and collection schedule |
| [`weather`](./modules/weather) | `ghcr.io/portakiapp/portaki-modules-weather:<semver>` | Current weather and 5-day forecast |
| [`wifi-guest`](./modules/wifi-guest) | `ghcr.io/portakiapp/portaki-modules-wifi-guest:<semver>` | Guest Wi-Fi SSID and password with timed reveal |

## Structure

```
portaki-modules/
├── Cargo.toml                 # workspace + shared SDK git deps (portaki-sdk main / 2.1+)
├── modules/
│   ├── access-guide/          # each crate: ids.rs, guest/, host/, …
│   ├── appliances/
│   ├── checklist/
│   ├── emergency-contacts/
│   ├── ev-parking/
│   ├── events/
│   ├── facility-hours/
│   ├── guest-reviews/
│   ├── ical-sync/
│   ├── issue-report/
│   ├── local-guide/
│   ├── lost-found/
│   ├── nuki/
│   ├── pre-arrival-form/
│   ├── rules/
│   ├── sections/
│   ├── train/
│   ├── waste-recycling/
│   ├── weather/
│   └── wifi-guest/
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
