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

1. Create `modules/<module-id>/` with `Cargo.toml`, `src/`, `i18n/`, and `tests/`.
2. Depend on workspace SDK crates (`portaki-sdk`, `portaki-connectors`, …).
3. Annotate the crate with `#[portaki_module(id = "…")]` in `lib.rs`.
4. Add per-module `.cargo/config.toml` with `target-dir = "target"` so `portaki build` / `portaki lint` find macro emissions (workspace builds otherwise use the repo-root `target/`).
5. Run quality gates from the repo root:

   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cd modules/<module-id> && portaki build --release && portaki lint
   ```

6. Open a PR to **`main`**. Do not push directly to `main`.

## Capability IDs

Prefer **string literals** in `#[capability]` attributes so proc-macro path resolution stays predictable.

## i18n

All user-facing copy must go through `i18n:` keys in bundles under `i18n/`. No hard-coded locale strings in Rust sources.

## Publishing

- Bump `version` in `modules/<module-id>/Cargo.toml`, then merge to **`main`**.
- CI publishes `ghcr.io/portakiapp/portaki-modules-<module-id>:<semver>` (`packages: write` via `GITHUB_TOKEN`).
- Module GHCR packages are **public**.
- Git tags / release-please — planned later.

## SDK dependency

The workspace pins `portaki-sdk` (and related crates) via git `branch = "main"` on [`PortakiApp/portaki-sdk`](https://github.com/PortakiApp/portaki-sdk).

Do **not** path-patch individual SDK crates into this workspace — that breaks `version.workspace` inheritance on SDK members.

## Pull requests

- Conventional Commits in English (`feat(weather): …`, `fix(ci): …`).
- Keep PRs focused; include a short test plan.
- CI must stay green.

## Security

Do not file public issues for vulnerabilities. See [SECURITY.md](./SECURITY.md).

## License

By contributing, you agree that your contributions are licensed under the [Apache License 2.0](./LICENSE).
