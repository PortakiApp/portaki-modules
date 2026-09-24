#!/usr/bin/env bash
# Les vingt et un modules dépendent du SDK du workspace, et d'aucun autre.
#
# Chaque module écrit la version de `portaki-sdk` dans son propre `Cargo.toml` au lieu de l'hériter
# du workspace. C'est ce qui rend une montée de SDK attribuable à chaque module par release-please
# — qui ne regarde que les chemins — donc publiable. Le prix est vingt-deux endroits au lieu d'un ;
# ce script est ce qui les tient ensemble. `portaki build` en tire `requiresModuleSdk`.
set -euo pipefail

cd "$(dirname "$0")/.."

pin="$(grep -oE '^portaki-sdk = "[^"]+"' Cargo.toml | head -1 | cut -d'"' -f2)"
if [[ -z "$pin" ]]; then
  echo "impossible de lire portaki-sdk dans Cargo.toml" >&2
  exit 1
fi

status=0
for cargo in modules/*/Cargo.toml; do
  declared="$(grep -oE '^portaki-sdk = \{ version = "[^"]+"' "$cargo" | cut -d'"' -f2 || true)"
  if [[ "$declared" != "$pin" ]]; then
    printf '%s dépend de portaki-sdk %s, le workspace épingle %s\n' \
      "$cargo" "${declared:-<absent>}" "$pin" >&2
    status=1
  fi
done

if [[ "$status" -ne 0 ]]; then
  echo "corrigez les modules, ou le pin — mais pas l'un sans l'autre" >&2
  exit 1
fi

echo "les $(ls -d modules/*/ | wc -l | tr -d ' ') modules dépendent du SDK $pin"
