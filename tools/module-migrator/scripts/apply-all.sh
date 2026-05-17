#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MODULES_ROOT="${PORTAKI_MODULES_DIR:-$ROOT/modules}"
DATABASE_URL="${DATABASE_URL:?Set DATABASE_URL}"

if command -v atlas >/dev/null 2>&1; then
  USE_ATLAS=1
else
  USE_ATLAS=0
  echo "atlas CLI not found — applying .sql files with psql in lexical order" >&2
fi

psql "$DATABASE_URL" -v ON_ERROR_STOP=1 <<'SQL'
CREATE TABLE IF NOT EXISTS t_e_module_db_migration (
    module_id VARCHAR(64) NOT NULL,
    revision VARCHAR(128) NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (module_id, revision)
);
SQL

for module_dir in "$MODULES_ROOT"/*/db/migrations; do
  [ -d "$module_dir" ] || continue
  module_id="$(basename "$(dirname "$(dirname "$module_dir")")")")"
  echo "== module $module_id =="

  if [ "$USE_ATLAS" = "1" ]; then
    atlas migrate apply --dir "file://$module_dir" --url "$DATABASE_URL" 2>/dev/null || true
  fi

  for file in "$module_dir"/*.sql; do
    [ -f "$file" ] || continue
    revision="$(basename "$file" .sql)"
    exists="$(psql "$DATABASE_URL" -tAc \
      "SELECT 1 FROM t_e_module_db_migration WHERE module_id='$module_id' AND revision='$revision'")"
    if [ "$exists" = "1" ]; then
      echo "  skip $revision"
      continue
    fi
    echo "  apply $revision"
    psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$file"
    psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -c \
      "INSERT INTO t_e_module_db_migration (module_id, revision) VALUES ('$module_id', '$revision')"
  done
done

echo "Done."
