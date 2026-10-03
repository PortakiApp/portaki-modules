#!/usr/bin/env bash
# Sortir du brouillon les versions déjà publiées, les vingt-trois d'un coup.
#
# Une stable publiée sans notes complètes reste en brouillon au registre : relisible, mais hors du
# catalogue et des mises à jour. Elle n'est pas republiable — `VERSION_ALREADY_PUBLISHED` refuse
# les mêmes octets sous le même numéro, et `DIGEST_ALREADY_PUBLISHED` sous un autre. Ce qui la
# débloque n'est donc pas une publication mais ses notes : `portaki release notes --complete`
# envoie celles de `CHANGELOG.fr.md` et `CHANGELOG.md` au registre, qui les revalide et la passe
# disponible. Sans montée de version, sans rien recompiler.
#
# À lancer depuis la machine de l'auteur : la route demande le compte de `portaki login`, pas le
# jeton OIDC d'un runner. Viser un environnement par `PORTAKI_API_URL`.
#
# Une fois les changelogs justes, la CI n'en a plus besoin : elle fournit les notes à la
# publication, et la version naît disponible. Ce script est pour l'arriéré.
set -euo pipefail

cd "$(dirname "$0")/.."

only="${1:-}"

./scripts/check-changelog-fr.sh

done_count=0
failed=()
for cargo in modules/*/Cargo.toml; do
  module="$(basename "$(dirname "$cargo")")"
  [[ -n "$only" && "$only" != "$module" ]] && continue
  version="$(sed -n 's/^version *= *"\(.*\)"/\1/p' "$cargo" | head -1)"

  printf '\n── %s %s\n' "$module" "$version"
  if portaki release notes "$version" --complete --module "$module" --plain; then
    done_count=$((done_count + 1))
  else
    failed+=("${module} ${version}")
  fi
done

printf '\n%s version(s) envoyée(s)' "$done_count"
if (( ${#failed[@]} )); then
  printf ', %s refusée(s) :\n' "${#failed[@]}"
  printf '  %s\n' "${failed[@]}"
  # Un refus porte ce qui manque encore (409 revalidé, rien écrit) : notes, justification de
  # permission, action de l'hôte, ou porte de conformité pas encore conclue.
  echo "le refus dit ce qui manque — rien n'a été écrit pour celles-là" >&2
  exit 1
fi
echo "."
