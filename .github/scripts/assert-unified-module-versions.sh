#!/usr/bin/env bash
# Vérifie que tous les package.json @portaki/module-* partagent la même version ; affiche la version sur stdout.
set -euo pipefail

FILES=(
  modules/*/package.json
  modules/pre-arrival-form/frontend/package.json
)

REF=""
for f in "${FILES[@]}"; do
  if [[ ! -f "$f" ]]; then
    continue
  fi
  name="$(jq -r .name "$f")"
  if [[ "$name" != @portaki/module-* ]]; then
    continue
  fi
  v="$(jq -r .version "$f")"
  if [[ -z "$REF" ]]; then
    REF="$v"
    continue
  fi
  if [[ "$v" != "$REF" ]]; then
    echo "::error::Version incohérente dans $f : $v (attendu $REF comme les autres modules)." >&2
    exit 1
  fi
done

if [[ -z "$REF" ]]; then
  echo "::error::Aucun package @portaki/module-* trouvé." >&2
  exit 1
fi

echo -n "$REF"
