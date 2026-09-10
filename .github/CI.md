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

## Dépendances — un seul gestionnaire

**Renovate écrit les PR. Dependabot ne les écrit plus.** Les deux tournaient : deux
gestionnaires ne font pas deux filets, ils font deux politiques, et celle qu'un second robot
ignore n'en est pas une.

Ce qui reste de Dependabot, et qui n'a rien à voir avec le fichier retiré : les **alertes de
vulnérabilité** et le **graphe de dépendances**, plus le **secret scanning** avec sa protection
au push — ce dépôt est public.

### Automerge

Patch, `pin` et `digest` fusionnent seuls quand `quality` est vert ; les majeures et les mineures
en `0.x` jamais. Toute montée attend **trois jours** après publication — sauf un correctif de
sécurité, où la version récente est justement ce qu'on veut.

La configuration vit dans `PortakiApp/renovate-config` ; ce dépôt n'en garde que ce qui lui est
propre, ci-dessous.

### Le SDK ne monte pas tout seul

`portaki-sdk` et ses trois crates voisines sont `"enabled": false` dans `renovate.json`. Ce
n'est pas de la prudence générale : `requiresModuleSdk` est **tamponné depuis le graphe résolu
par cargo**, donc une montée automatique changerait ce que ces vingt et un modules annoncent au
registre sans que personne ne l'ait décidé.

C'est exactement ce que 9a2a374 a corrigé en quittant `branch = "main"` pour une version fixe :
monter de SDK doit être un commit qu'on relit. La règle vivait dans le `dependabot.yml` sous
forme d'`ignore` ; le `renovate.json`, lui, ciblait `depTypes: ["git"]` — qui ne matche plus
rien depuis ce même commit. Retirer Dependabot sans porter la règle aurait donc rouvert la
dérive que 9a2a374 avait fermée.
