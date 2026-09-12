#!/usr/bin/env bash
# Les vingt et un manifestes annoncent le SDK du workspace, et rien d'autre.
#
# `requiresModuleSdk` est déclaré par chaque `portaki.module.json` et non plus déduit du graphe
# cargo à la publication. C'est ce qui rend une montée de SDK attribuable à chaque module par
# release-please — qui ne regarde que les chemins — donc publiable. Le prix est vingt-deux
# endroits au lieu d'un ; ce script est ce qui les tient ensemble.
#
# Le CLI refuse de publier un manifeste qui annonce un autre SDK que celui lié au build. Sans ce
# contrôle, la divergence n'apparaîtrait qu'à la publication, module par module.
set -euo pipefail

cd "$(dirname "$0")/.."

pin="$(grep -oE '^portaki-sdk = "[^"]+"' Cargo.toml | head -1 | cut -d'"' -f2)"
if [[ -z "$pin" ]]; then
  echo "impossible de lire portaki-sdk dans Cargo.toml" >&2
  exit 1
fi

status=0
for manifest in modules/*/portaki.module.json; do
  declared="$(python3 -c "
import json, sys
print(json.load(open('$manifest')).get('requiresModuleSdk', ''))")"
  if [[ "$declared" != "$pin" ]]; then
    printf '%s annonce requiresModuleSdk %s, le workspace epingle %s\n' \
      "$manifest" "${declared:-<absent>}" "$pin" >&2
    status=1
  fi
done

if [[ "$status" -ne 0 ]]; then
  echo "corrigez les manifestes, ou le pin — mais pas l'un sans l'autre" >&2
  exit 1
fi

echo "les $(ls -d modules/*/ | wc -l | tr -d ' ') manifestes annoncent le SDK $pin"
