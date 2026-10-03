#!/usr/bin/env bash
# Chaque module décrit sa version déclarée en français, dans les bornes du registre.
#
# Le registre garde une version stable en brouillon tant que ses notes ne couvrent pas toutes les
# langues de sa fiche (`ReleaseState`), et le français en est de toutes. Rien ne rougit : la
# publication réussit, la version n'entre simplement jamais au catalogue. Les vingt-trois modules
# y sont restés d'octobre, faute d'une section `## <version>` dans `CHANGELOG.fr.md`.
#
# Ce que `changelog.rs` lit, et donc ce qu'on contrôle ici : la section de la version déclarée dans
# `Cargo.toml`, ses puces `* ` seules (une puce coupée sur deux lignes perd sa fin), cinq au plus,
# cent soixante caractères au plus, et pas une ligne qui ressemble à un message de commit.
set -euo pipefail

cd "$(dirname "$0")/.."

# Les bornes de `contracts/release-notes-rules.json`, côté SDK.
max_lines=5
max_chars=160
types='feat|fix|chore|refactor|deps|build|ci|perf|test|tests|docs|doc|style|revert|release'
words='bump|chore|refactor|deps|wip|merge branch|merge pull request'

status=0
for cargo in modules/*/Cargo.toml; do
  module="$(basename "$(dirname "$cargo")")"
  changelog="modules/${module}/CHANGELOG.fr.md"
  version="$(sed -n 's/^version *= *"\(.*\)"/\1/p' "$cargo" | head -1)"

  if [[ ! -f "$changelog" ]]; then
    printf '%s : CHANGELOG.fr.md manque\n' "$module" >&2
    status=1
    continue
  fi

  # La section de cette version, jusqu'au titre suivant ; `## [1.2.0](…)` compte aussi.
  bullets="$(awk -v version="$version" '
    /^## /  { if (inside) exit; inside = (index($0, version) > 0); next }
    inside && /^\* / { print substr($0, 3) }
  ' "$changelog")"

  if [[ -z "$bullets" ]]; then
    printf '%s : CHANGELOG.fr.md ne porte aucune puce sous « ## %s »\n' "$module" "$version" >&2
    status=1
    continue
  fi

  count="$(printf '%s\n' "$bullets" | wc -l | tr -d ' ')"
  if (( count > max_lines )); then
    printf '%s : %s puces sous « ## %s », %s au plus\n' "$module" "$count" "$version" "$max_lines" >&2
    status=1
  fi

  while IFS= read -r line; do
    if (( "$(printf '%s' "$line" | wc -m | tr -d ' ')" > max_chars )); then
      printf '%s : puce de plus de %s caractères — « %s »\n' "$module" "$max_chars" "$line" >&2
      status=1
    fi
    lowered="$(printf '%s' "$line" | tr '[:upper:]' '[:lower:]')"
    if [[ "$lowered" =~ ^($types)(\([^\)]*\))?!?: || "$lowered" =~ ^($words)([^[:alnum:]]|$) ]]; then
      printf '%s : puce lue comme un message de commit, le registre la jette — « %s »\n' \
        "$module" "$line" >&2
      status=1
    fi
  done <<<"$bullets"
done

if (( status != 0 )); then
  echo "écrivez la section française de la version : sans elle, la stable reste en brouillon" >&2
  exit 1
fi

echo "CHANGELOG.fr.md : chaque module décrit sa version déclarée."
