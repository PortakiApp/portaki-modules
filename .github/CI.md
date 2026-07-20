# GitHub Actions CI

- Several jobs per quality workflow — never one monolith.
- **One job = one subject** (`fmt`, `clippy`, `test`, `wasm`, `publish`, …).
- Final **`quality` gate** (`if: always()`) aggregates job results.
- Shared build: producer uploads artifact; consumers download (no rebuild).
- Artifacts: `retention-days: 1` (GitHub minimum). Stable names; `if-no-files-found: error` when required.
- Multi-module work: **matrix** (`wasm` → artifact `wasm-{module}` → `publish` on `main`).

Local Cursor mirror (gitignored): `.cursor/rules/github-actions-ci.mdc`.
