<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://portaki.app/logo-dark.svg">
    <img src="https://portaki.app/logo-light.svg" width="177" height="48" alt="Portaki">
  </picture>
</p>

<h1 align="center">portaki-modules</h1>

<p align="center">
  <strong>The official Portaki modules</strong><br>
  Rust crates compiled to Wasm (Extism), versioned one by one and published to Portaki's OCI registry.
</p>

<p align="center">
  <a href="https://github.com/PortakiApp/portaki-modules/actions/workflows/ci.yml"><img src="https://github.com/PortakiApp/portaki-modules/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue.svg" alt="License Apache-2.0"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.75+-dea584?logo=rust&logoColor=white" alt="Rust 1.75+"></a>
</p>

<p align="center">
  <a href="#modules">Modules</a> ·
  <a href="#getting-started">Getting started</a> ·
  <a href="#before-opening-a-pr">Before a PR</a> ·
  <a href="#releasing">Releasing</a> ·
  <a href="CONTRIBUTING.md">Contributing</a> ·
  <a href="SECURITY.md">Security</a>
</p>

---

[Portaki](https://portaki.app) is a digital welcome booklet for short-term rental hosts. Everything the
guest sees in the booklet — Wi-Fi, access steps, house rules, weather, local guide… — and the matching
host settings is a **module**: a sandboxed Wasm plugin that renders server-driven UI and talks to the
platform only through the permissions it declares.

This repository holds the modules Portaki ships itself. They are written with
[`portaki-sdk`](https://github.com/PortakiApp/portaki-sdk) and its `portaki` CLI — the same SDK a
third-party module uses from its own repository. Read them as working examples.

Each module is published as `oci.portaki.app/modules/<module-id>:<semver>`.

## Modules

23 modules, one crate each under [`modules/`](./modules):

| Module | What it does |
|--------|--------------|
| [`access-guide`](./modules/access-guide) | Arrival steps, codes, and parking |
| [`appliances`](./modules/appliances) | Device guides and safety notice |
| [`checklist`](./modules/checklist) | Checkout checklist with guest toggles |
| [`consumables`](./modules/consumables) | Host stock list and guest shortage reports |
| [`emergency-contacts`](./modules/emergency-contacts) | Useful numbers and host line |
| [`ev-parking`](./modules/ev-parking) | EV spot, gate code, charger PIN with timed reveal |
| [`events`](./modules/events) | Local happenings near the property |
| [`facility-hours`](./modules/facility-hours) | Pool, spa, and shared amenity schedules |
| [`guest-reviews`](./modules/guest-reviews) | Post-stay thank-you and review CTAs |
| [`ical-sync`](./modules/ical-sync) | Host calendar feed import (iCal / Airbnb) |
| [`issue-report`](./modules/issue-report) | In-stay problem reports for the host |
| [`local-guide`](./modules/local-guide) | Nearby spots and host picks |
| [`lost-found`](./modules/lost-found) | Guest lost / found item reports |
| [`nuki`](./modules/nuki) | Nuki smart-lock provider for `access-guide` |
| [`pre-arrival-form`](./modules/pre-arrival-form) | ETA, occasion, allergies, message to host |
| [`rules`](./modules/rules) | Structured house rules |
| [`safety-shutoffs`](./modules/safety-shutoffs) | Where to cut power, water and gas |
| [`sections`](./modules/sections) | Editorial title + markdown body blocks |
| [`trails`](./modules/trails) | Walks around the property, by level |
| [`train`](./modules/train) | Nearest station departure board |
| [`waste-recycling`](./modules/waste-recycling) | Bins and collection schedule |
| [`weather`](./modules/weather) | Current conditions and 5-day forecast |
| [`wifi-guest`](./modules/wifi-guest) | Guest Wi-Fi SSID and password with timed reveal |

Catalogue order and maturity: [docs/MARKETPLACE_ORDER.md](./docs/MARKETPLACE_ORDER.md).
What the guest sees, module by module (French): [docs/guest-surfaces.md](./docs/guest-surfaces.md).

## Layout

```
portaki-modules/
├── Cargo.toml                  # workspace, pins portaki-sdk (crates.io)
├── modules/<module-id>/
│   ├── Cargo.toml              # version + portaki-sdk with the features (permissions) it uses
│   ├── src/                    # guest/, host/, connectors — see docs/module-layout.md
│   ├── i18n/                   # every user-facing text
│   ├── tests/                  # conformance + scenarios
│   ├── listing.json            # public catalogue listing (fr + en)
│   ├── permission-reasons.json # why each permission is asked, shown to the host
│   ├── CHANGELOG.md            # written by release-please
│   └── CHANGELOG.fr.md         # French release notes, written by hand
├── support/                    # test helpers shared by the modules
├── scripts/                    # check-sdk-pin.sh, release-please config generator, …
└── .github/workflows/          # ci, budget, release-please, security-audit
```

There is no hand-written manifest: `portaki build` derives it from the macros
(`portaki_module!`, `#[surface]`, `#[query]`, `#[command]`…), the SDK features and `i18n/`.

## Getting started

Requirements: Rust (stable, via [`rust-toolchain.toml`](./rust-toolchain.toml)), the
`wasm32-unknown-unknown` target, and the `portaki` CLI at the version the workspace pins
(`portaki-sdk` in [`Cargo.toml`](./Cargo.toml)).

```bash
git clone https://github.com/PortakiApp/portaki-modules.git
cd portaki-modules
rustup target add wasm32-unknown-unknown
cargo install portaki-cli --version <pinned portaki-sdk version> --locked
```

Workspace checks, from the repo root:

```bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --tests
```

One module, from its directory — **`portaki build` first**, so `portaki check` reads a fresh manifest
rather than a stale one:

```bash
cd modules/weather
portaki build
portaki check
```

`portaki dev`, `portaki preview` and `portaki scenarios` run a module against Portaki's hosted
sandbox (needs `portaki login`). `portaki --help` lists the rest.

Adding a module (`portaki init`, release-please registration, per-module `.cargo/config.toml`…) is
covered in [CONTRIBUTING.md](./CONTRIBUTING.md).

## Before opening a PR

The `ci` workflow does **not** run on pull requests — only `budget` does. Lint, tests and the publish
gates run on `main`, after the merge. Run them locally first:

```bash
./scripts/check-sdk-pin.sh                  # every module's portaki-sdk = the workspace pin
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --tests
cd modules/<module-id> && portaki build && portaki check && cd -
portaki ci check --module <module-id>       # release notes in every language, permission reasons, entity columns, SDK pin
```

`portaki ci check` replaces the old `check-changelog-fr.sh` / `check-permission-reasons.sh`
scripts; it runs in CI inside the module build, after `portaki build`.

Commits follow Conventional Commits scoped to the module (`feat(weather): …`, `fix(rules): …`) —
release-please uses the scope's path to decide what to bump.

## Releasing

Versions are owned by **release-please**, one package per `modules/<id>/`. Do not bump by hand.

1. Conventional commits land on `main`.
2. The `Release please` workflow opens a release PR that bumps each touched module's `Cargo.toml`
   and `CHANGELOG.md`.
3. **Write the French notes** in each bumped module's `CHANGELOG.fr.md`, on the release PR, before
   merging it. They are mandatory: `portaki ci check` fails the build without them, and a red build
   skips publishing for **every** module of that release, not only the one missing its notes.
4. Merge the release PR. The `ci` workflow publishes only the modules whose version changed.

Publishing runs in two jobs: the module is built and tested in a job without rights
(`portaki-release-action/build`), then a job holding only `id-token: write` pushes it to the
registry, attests the digest (workflow provenance + `cargo audit`) and announces it. Official modules
publish from CI only. `ci` can also be dispatched by hand with `module=<id>` or `module=all`.

Pipeline details: [.github/CI.md](./.github/CI.md).

### What CI skips

`ci` ignores pushes that only touch `**.md`, `docs/**` or `.github/CI.md` — including the
`CHANGELOG*.md` files. Fixing notes after a release was skipped publishes nothing on its own:
dispatch `ci` with the affected module ids (or `all`). A version already published as a draft
because its notes were incomplete is completed with
[`scripts/complete-releases.sh`](./scripts/complete-releases.sh), without republishing.

## Related repositories

| Repository | Role |
|------------|------|
| [portaki-sdk](https://github.com/PortakiApp/portaki-sdk) | Rust SDK, `portaki` CLI, SDUI catalogue |
| [portaki-release-action](https://github.com/PortakiApp/portaki-release-action) | GitHub Action that builds and publishes a module |
| [portaki.app](https://portaki.app) | Product site |

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) and the [Code of Conduct](./CODE_OF_CONDUCT.md).

Security issues: [SECURITY.md](./SECURITY.md) — do not open a public issue.

## License

[Apache-2.0](./LICENSE) · Copyright 2026 Syntax Labs
