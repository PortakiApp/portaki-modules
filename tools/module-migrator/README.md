# portaki-module-migrator

Applies **versioned DDL** owned by each catalogue module (`portaki-modules/modules/<id>/db/migrations/`).

## Principles

- One migration history row per `(module_id, revision)` in `t_e_module_db_migration` (core DB).
- **No DDL from gateway handlers** — only this job / API startup runner / CI.
- **Atlas** validates and plans migrations locally; runtime applies ordered SQL from module backend JARs (`classpath:portaki/db/<moduleId>/`).

## Local (Atlas CLI)

```bash
export DATABASE_URL="postgres://portaki:portaki@localhost:5432/portaki?sslmode=disable"

# Lint a module
cd ../portaki-modules/modules/sections/db
atlas migrate lint --dir file://migrations

# Apply one module (dev)
atlas migrate apply --dir file://migrations --url "$DATABASE_URL"
```

Install Atlas: https://atlasgo.io/getting-started

## Apply all modules (shell)

```bash
./scripts/apply-all.sh
```

Requires `psql` or `DATABASE_URL`. Used in CI before integration tests.

## Production

`portaki-api` runs `ModuleDatabaseMigrationRunner` on startup when `portaki.modules.db-migrate=true` (default in dev profile).

See `portaki-internal-docs/MODULE_PLATFORM_PREPARATION.md`.
