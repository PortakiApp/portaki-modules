# GitHub Actions CI

## Flow

1. **`changes`** — `portaki ci modules` → JSON module list (or empty).
2. **`rust`** — fmt + clippy + tests on one runner (setup cost shared).
3. **`wasm` matrix** (only if modules changed) → upload `wasm-{module}` artifact.
4. **`publish` matrix** on `main` push — download artifact, `portaki-release-action` with `build: false`.
5. **`quality`** gate aggregates results.

## Changed-modules rules

- Touch `modules/<name>/…` → only that module’s wasm/publish.
- Touch `Cargo.toml` / `Cargo.lock` / toolchain → **all** modules.
- Touch **only** `.github/workflows/` or `.github/scripts/` → **no** wasm matrix (rust still runs). CI-only edits must not burn N× publish minutes.

## Minutes

GitHub bills **job-minutes**. Parallel matrix jobs multiply cost.

- Quality on **`pull_request`**; `push` only on **`main`** (publish) — no `feature/**` double bill with an open PR.
- `max-parallel: 2` on wasm/publish — softens spikes without serializing forever.
- Prefer one rust job over fmt/clippy/test split when setup dominates.
- `concurrency` cancels superseded PR runs.
- Artifacts: `retention-days: 1`.

Local Cursor mirror (gitignored): `.cursor/rules/github-actions-ci.mdc`.

## Release please

Workflow: [`.github/workflows/release-please.yml`](./workflows/release-please.yml).

### Dynamic packages

`scripts/generate-release-please-config.sh` scans `modules/*/Cargo.toml` + `portaki.module.json` and writes:

- `release-please-config.json` — one package path per module (`modules/<id>`)
- `.release-please-manifest.json` — last released versions (new modules seeded from Cargo.toml)

Do not maintain a static list of module ids by hand. The generator is the source of truth for package paths.

`googleapis/release-please-action@v5` loads those files via the **GitHub API** from the tip of `main` (local checkout alone is ignored). The workflow therefore:

1. Regenerates the two files on every run
2. Commits and pushes if they drifted (e.g. a new module landed without regenerating)
3. Runs release-please against the tip

When adding a module locally, still run the script in the PR so review sees the config.

### Token (CI App)

Requires repository secrets `CI_APP_ID` and `CI_APP_PRIVATE_KEY` (same App as dashboard/sdk).

Plain `GITHUB_TOKEN` limitations if you ever fall back:

- May lack permission to open/update release PRs depending on org defaults
- Commits/tags created with `GITHUB_TOKEN` **do not** trigger other workflows — so a merged release PR would not run `ci` publish

### After merge

Release PR merge bumps module versions on `main` → existing `ci` `publish` matrix builds/publishes GHCR from Cargo.toml. Tags / GitHub Releases are optional extras from release-please.
