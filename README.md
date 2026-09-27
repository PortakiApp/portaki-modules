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
  <a href="https://portaki.app"><img src="https://img.shields.io/badge/site-portaki.app-f59e0b" alt="portaki.app"></a>
</p>

<p align="center">
  <a href="#modules">Modules</a> ·
  <a href="#requirements">Requirements</a> ·
  <a href="#development">Development</a> ·
  <a href="#publishing">Publishing</a> ·
  <a href="docs/MARKETPLACE_ORDER.md">Marketplace order</a> ·
  <a href="CONTRIBUTING.md">Contributing</a> ·
  <a href="SECURITY.md">Security</a>
</p>

---

Each crate under `modules/` is a Portaki guest module. Authoring uses [`portaki-sdk`](https://github.com/PortakiApp/portaki-sdk) (`portaki` CLI).

On every push to **`main`**, CI builds and publishes:

`oci.portaki.app/modules/<module-id>:<semver>`

## Why this monorepo?

- **One repo per ecosystem** — shared CI, shared SDK pins, consistent lint/build gates
- **Independent versions** — each module bumps its own `Cargo.toml` semver
- **OCI-first** — pushed to Portaki's OCI registry (`oci.portaki.app/modules/weather`, …), signed by CI
- **Pattern A** — official modules live here; third-party modules use standalone repos + the same SDK

## Modules

| Module | OCI image | Description |
|--------|-----------|-------------|
| [`access-guide`](./modules/access-guide) | `oci.portaki.app/modules/access-guide:<semver>` | Arrival steps, codes, and parking |
| [`appliances`](./modules/appliances) | `oci.portaki.app/modules/appliances:<semver>` | Device guides and safety notice |
| [`checklist`](./modules/checklist) | `oci.portaki.app/modules/checklist:<semver>` | Checkout checklist with guest toggles |
| [`consumables`](./modules/consumables) | `oci.portaki.app/modules/consumables:<semver>` | Consumables catalog, guest shortages, restock tracking |
| [`emergency-contacts`](./modules/emergency-contacts) | `oci.portaki.app/modules/emergency-contacts:<semver>` | Useful numbers and host line |
| [`ev-parking`](./modules/ev-parking) | `oci.portaki.app/modules/ev-parking:<semver>` | EV spot, gate code, charger PIN with timed reveal |
| [`events`](./modules/events) | `oci.portaki.app/modules/events:<semver>` | Host-curated local events and map |
| [`facility-hours`](./modules/facility-hours) | `oci.portaki.app/modules/facility-hours:<semver>` | Pool, spa, and shared amenity schedules |
| [`guest-reviews`](./modules/guest-reviews) | `oci.portaki.app/modules/guest-reviews:<semver>` | Post-stay thank-you and review CTAs |
| [`ical-sync`](./modules/ical-sync) | `oci.portaki.app/modules/ical-sync:<semver>` | Host iCal / Airbnb calendar feed import |
| [`issue-report`](./modules/issue-report) | `oci.portaki.app/modules/issue-report:<semver>` | In-stay problem reports for the host |
| [`local-guide`](./modules/local-guide) | `oci.portaki.app/modules/local-guide:<semver>` | Nearby spots and host picks |
| [`lost-found`](./modules/lost-found) | `oci.portaki.app/modules/lost-found:<semver>` | Guest lost / found item reports |
| [`nuki`](./modules/nuki) | `oci.portaki.app/modules/nuki:<semver>` | Nuki smart-lock provider for access-guide |
| [`pre-arrival-form`](./modules/pre-arrival-form) | `oci.portaki.app/modules/pre-arrival-form:<semver>` | ETA, occasion, allergies, message to host |
| [`rules`](./modules/rules) | `oci.portaki.app/modules/rules:<semver>` | Structured bilingual house rules |
| [`sections`](./modules/sections) | `oci.portaki.app/modules/sections:<semver>` | Editorial title + markdown body blocks |
| [`train`](./modules/train) | `oci.portaki.app/modules/train:<semver>` | Nearby station departure board |
| [`waste-recycling`](./modules/waste-recycling) | `oci.portaki.app/modules/waste-recycling:<semver>` | Bins and collection schedule |
| [`weather`](./modules/weather) | `oci.portaki.app/modules/weather:<semver>` | Current weather and 5-day forecast |
| [`wifi-guest`](./modules/wifi-guest) | `oci.portaki.app/modules/wifi-guest:<semver>` | Guest Wi-Fi SSID and password with timed reveal |

## Structure

```
portaki-modules/
├── Cargo.toml                 # workspace + shared SDK git deps (portaki-sdk main / 2.1+)
├── modules/
│   ├── access-guide/          # each crate: guest/, host/, …
│   ├── appliances/
│   ├── checklist/
│   ├── consumables/
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
    └── ci.yml                 # quality gates; publish to Portaki on main
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
3. CI publishes `oci.portaki.app/modules/<id>:<semver>`

CI builds each module in a job without rights (`portaki-release-action/build@v2`), then publishes from a job holding only `id-token: write` (`portaki-release-action@v2`): the registry grants a short push right to its own OCI repository, the digest is attested with this workflow's provenance and `cargo audit` report, then announced. Official modules publish from CI only.

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
