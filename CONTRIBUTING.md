# Contributing to portaki-modules

This monorepo hosts **official** Portaki modules. Third-party modules should live in standalone repositories and depend on [`portaki-sdk`](https://github.com/PortakiApp/portaki-sdk).

## Development setup

```bash
git clone https://github.com/PortakiApp/portaki-modules.git
cd portaki-modules
rustup target add wasm32-unknown-unknown
cargo install --git https://github.com/PortakiApp/portaki-sdk --branch main --locked portaki-cli
```

## Adding a module

1. Create `modules/<module-id>/` with `Cargo.toml`, `src/`, `i18n/`, and `tests/` — `portaki init`
   does it. There is no manifest to write: `portaki build` derives it from `portaki_module!`,
   `#[surface]`, `#[email]`, `#[nav]`, the `portaki-sdk` features (the permissions) and `i18n/`.
   `tests/conformance.rs` holds `portaki_test_utils::conformance!();` — the SDK's conformance
   battery, which `portaki publish` refuses to publish without. `tests/scenarios.rs` includes
   `support/scenarios.rs`: every surface on the sandbox's seven pathological stays, and every
   `example(label = …, input = …)` of the queries and commands — declare one on each.
2. The version lives in `Cargo.toml` only. Add `listing.json` next to it: the public catalogue listing, published by CI with each release.
3. Depend on `portaki-sdk` at the workspace's pinned version, written in the module's own
   `Cargo.toml` with the features it uses (`kv`, `repo`, `email`, `events`, `platform`,
   `guest-files`…) — `scripts/check-sdk-pin.sh` keeps the pins together. The other SDK crates
   come from the workspace.
4. Follow the Wasm crate layout: `guest/`, `host/` (omit if guest-only), `connectors` when
   needed, and `ids.rs` only for `define_event_types!` / `module_id()`.
   `#[surface(id = "home.card")]` declares `HOME_CARD`, `#[query(name = "statsSummary")]` /
   `#[command]` declare `STATS_SUMMARY`, next to the handler — no `define_surface_ids!` /
   `define_operation_names!` (deprecated in SDK 8.7.0).
   Boundary builders (`Action::command` / `open_overlay` / `emit` / `navigate`,
   `Surface::with_id`, `events::emit`) take those typed consts (or `contracts::*`) — no bare
   `"home.card"` / `"updateConfig"` at use sites. Declaration sites (`define_event_types!`,
   `#[surface]` / `#[command]` / `#[query]` / `#[event_handler]`) may use literals once —
   macros cannot take `ids::CONST` paths (emission needs the wire string at expand).
5. Annotate the crate with `#[portaki_module(id = "…")]` in `lib.rs`.
6. Add per-module `.cargo/config.toml` with `target-dir = "target"` so `portaki build` / `portaki lint` find macro emissions (workspace builds otherwise use the repo-root `target/`).
7. Regenerate release-please package discovery (required — do not hand-edit package paths):

   ```bash
   ./scripts/generate-release-please-config.sh
   ```

   Commit the updated `release-please-config.json` and `.release-please-manifest.json` in the same PR. The release-please workflow also regenerates on `main` and commits drift if a module was added without running the script.
8. Run quality gates from the repo root:

   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cd modules/<module-id> && portaki build --release && portaki lint
   ```

9. Open a PR to **`main`**. Do not push directly to `main`.

## Capability IDs

Prefer **string literals** in `#[capability]` attributes so proc-macro path resolution stays predictable.

## i18n

All user-facing copy must go through `i18n:` keys in bundles under `i18n/`. No hard-coded locale strings in Rust sources.

## Publishing

Versions are managed by **release-please** (multi-package, one entry per `modules/<id>/` crate).

1. Land conventional commits on **`main`** scoped to a module (`feat(access-guide): …`, `fix(weather): …`).
2. The `Release please` workflow opens/updates a draft release PR that bumps:
   - `modules/<id>/Cargo.toml` `package.version`
   - `modules/<id>/CHANGELOG.md`
   - `.release-please-manifest.json`
3. Merge the release PR. That push to `main` runs existing `ci` publish (GHCR) from the bumped Cargo.toml version.
4. release-please also creates GitHub Releases / tags (`<module-id>-vX.Y.Z`) — nice-to-have; GHCR is the publish path that matters.

Do **not** hand-bump versions for routine releases. Do **not** hand-edit package paths in `release-please-config.json` — run `./scripts/generate-release-please-config.sh` instead.

Details: [`.github/CI.md`](./.github/CI.md).

## SDK dependency

The workspace pins `portaki-sdk` (and related crates) via git `branch = "main"` on
[`PortakiApp/portaki-sdk`](https://github.com/PortakiApp/portaki-sdk).

Requires **portaki-sdk ≥ 2.1.0** (typed boundary ids — `SurfaceId`, `OperationName`,
`EventType`, `ModuleId`, `define_*!` catalogs). See
[docs/module-layout.md](./docs/module-layout.md) and the SDK docs
`typed-ids.md` / `module-layout.md`.

Do **not** path-patch individual SDK crates into this workspace for CI — that
breaks remote builds. Use a local path override only in a private
`.cargo/config.toml` (gitignored) if needed.

## Pull requests

- Conventional Commits in English (`feat(weather): …`, `fix(ci): …`).
- Keep PRs focused; include a short test plan.
- CI must stay green.

## Security

Do not file public issues for vulnerabilities. See [SECURITY.md](./SECURITY.md).

## License

By contributing, you agree that your contributions are licensed under the [Apache License 2.0](./LICENSE).
