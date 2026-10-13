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
#
# Une version qui ne fait que monter une dépendance (`fix(deps): bump portaki-sdk to …`, que
# release-please pose sur les vingt-trois modules) peut se passer de section française : quand la
# section de `CHANGELOG.md` ne porte que des montées de dépendances, la CLI publie une ligne
# standard. Même règle que `changelog.rs` : chaque puce se lit comme un commit, et c'est une
# portée `**deps:**` ou un `bump <crate> to <version>`. Une vraie ligne, et la section reste due.
#
# `--self-test` rejoue le contrôle sur des modules factices.
set -euo pipefail

if [[ "${1:-}" == "--self-test" ]]; then
  script="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  fixture() { # nom, CHANGELOG.md, CHANGELOG.fr.md
    mkdir -p "$tmp/$1/modules/nuki"
    printf '[package]\nname = "nuki"\nversion = "1.8.1"\n' >"$tmp/$1/modules/nuki/Cargo.toml"
    printf '%b' "$2" >"$tmp/$1/modules/nuki/CHANGELOG.md"
    printf '%b' "$3" >"$tmp/$1/modules/nuki/CHANGELOG.fr.md"
  }
  head='# Changelog\n\n## [1.8.1](https://github.com/PortakiApp/portaki-modules/compare/nuki-v1.8.0...nuki-v1.8.1) (2026-10-13)\n\n\n### Bug Fixes\n\n'
  bump='* **deps:** bump portaki-sdk to 10.10.0 ([f9906b0](https://github.com/PortakiApp/portaki-modules/commit/f9906b0))\n'
  older='## 1.8.0\n\n* Batterie de la serrure affichée\n'
  fixture deps-only "$head$bump" "# Changelog\n\n$older"
  fixture bare-bump "## 1.8.1\n- Bump portaki-sdk to v10.10.0\n" ''
  fixture written "$head$bump" "## 1.8.1\n\n* Code clavier la veille\n\n$older"
  fixture real-change "$head$bump* **nuki:** show the battery ([1a2b3c4](https://x))\n" "$older"
  fixture other-commit "$head$bump* chore: release 1.8.1\n" "$older"
  fixture deps-prose "$head* **deps:** build against portaki-sdk 10.10\n" "$older"
  fixture no-section "## 1.8.0\n* **deps:** bump portaki-sdk to 10.9.0\n" "$older"
  fixture near-version "## 1.8.10\n* **deps:** bump portaki-sdk to 10.10.0\n" "$older"
  failed=0
  for case in deps-only:0 bare-bump:0 written:0 real-change:1 other-commit:1 deps-prose:1 \
    no-section:1 near-version:1; do
    name="${case%%:*}"
    want="${case##*:}"
    got=0
    CHANGELOG_FR_ROOT="$tmp/$name" "$script" >/dev/null 2>&1 || got=1
    if [[ "$got" != "$want" ]]; then
      printf 'self-test %s : sortie %s, attendu %s\n' "$name" "$got" "$want" >&2
      failed=1
    fi
  done
  (( failed == 0 )) && echo "check-changelog-fr.sh : self-test réussi."
  exit "$failed"
fi

cd "${CHANGELOG_FR_ROOT:-$(dirname "$0")/..}"

# Les bornes de `contracts/release-notes-rules.json`, côté SDK.
max_lines=5
max_chars=160
types='feat|fix|chore|refactor|deps|build|ci|perf|test|tests|docs|doc|style|revert|release'
words='bump|chore|refactor|deps|wip|merge branch|merge pull request'

# La section `## <version>` de `CHANGELOG.md` ne porte que des montées de dépendances.
deps_only() { # fichier, version
  local bullets line scope lowered found=0
  [[ -f "$1" ]] || return 1
  # Le titre exact : `## [1.8.1](…)`, `## 1.8.1`, `## v1.8.1 - …` — pas `## 1.8.10`.
  bullets="$(awk -v version="$2" '
    /^## / {
      if (inside) exit
      token = substr($0, 4)
      if (substr(token, 1, 1) == "[") { sub(/^\[/, "", token); sub(/\].*/, "", token) }
      else { sub(/[ \t].*/, "", token) }
      sub(/^v/, "", token)
      inside = (token == version)
      next
    }
    inside && /^[*-] / { print substr($0, 3) }
  ' "$1")"
  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    found=1
    line="${line%% (\[*}"
    scope=""
    if [[ "$line" =~ ^\*\*([^:]*):\*\*[[:space:]]*(.*)$ ]]; then
      scope="${BASH_REMATCH[1]}"
      line="${BASH_REMATCH[2]}"
    fi
    lowered="$(printf '%s' "$line" | tr '[:upper:]' '[:lower:]')"
    commit_like "$lowered" || return 1
    [[ "$scope" == "deps" ]] && continue
    [[ "$lowered" =~ ^bump\ [^[:space:]]+\ to\ v?[0-9][[:alnum:].+-]*$ ]] || return 1
  done <<<"$bullets"
  (( found ))
}

commit_like() { # ligne en minuscules
  [[ "$1" =~ ^($types)(\([^\)]*\))?!?: || "$1" =~ ^($words)([^[:alnum:]]|$) ]]
}

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
    /^## /  {
      if (inside) exit
      token = substr($0, 4)
      if (substr(token, 1, 1) == "[") { sub(/^\[/, "", token); sub(/\].*/, "", token) }
      else { sub(/[ \t].*/, "", token) }
      sub(/^v/, "", token)
      inside = (token == version)
      next
    }
    inside && /^\* / { print substr($0, 3) }
  ' "$changelog")"

  if [[ -z "$bullets" ]] && deps_only "modules/${module}/CHANGELOG.md" "$version"; then
    printf '%s : %s ne monte que des dépendances, la CLI publiera la ligne standard\n' \
      "$module" "$version"
    continue
  fi

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
    if commit_like "$lowered"; then
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
