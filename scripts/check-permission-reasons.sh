#!/usr/bin/env bash
# Chaque permission et chaque secret révélé que déclare le manifeste construit d'un module est
# justifié, en français et en anglais, dans son `permission-reasons.json`.
#
# Le registre garde une stable en brouillon tant qu'une permission ajoutée depuis la précédente
# n'est pas justifiée dans chaque langue de sa fiche. La CI ne passe aucun drapeau : `portaki
# release` lit ces justifications dans le fichier, et n'envoie que les clés que déclare le manifeste.
# Une clé absente ne rougit rien — la version reste invisible des hôtes.
#
# Ce qu'on contrôle, d'après `changelog.rs` (`declared`, `file_reasons`) : les clés de
# `permissions`, plus `reveal:<champ>:<lieu>` par lieu de chaque secret révélé (`<liste>[].<sous-clé>`
# pour un secret de ligne) ; chacune présente avec `fr` et `en`, de 1 à 160 caractères.
#
# Lit `target/portaki/publish-manifest.json` : lancer `portaki build` d'abord. Sans argument, tous
# les modules ; sinon ceux qu'on nomme.
set -euo pipefail

cd "$(dirname "$0")/.."

max_chars=160

if (( $# > 0 )); then
  modules=("$@")
else
  modules=()
  for dir in modules/*/; do modules+=("$(basename "$dir")"); done
fi

status=0
for module in "${modules[@]}"; do
  manifest="modules/${module}/target/portaki/publish-manifest.json"
  reasons="modules/${module}/permission-reasons.json"

  if [[ ! -f "$manifest" ]]; then
    printf '%s : %s manque — lancez portaki build\n' "$module" "$manifest" >&2
    status=1
    continue
  fi

  declared="$(jq -r '
    [(.permissions // [])[]]
    + [(.config.fields // [])[] | select(.key) as $field
       | (if .type == "secret" then [.key]
          elif .type == "structured" then [(.item.secret // [])[] | "\($field.key)[].\(.)"]
          else [] end) as $secrets
       | (.reveal // [])[]
       | select(. == "guest_pre_arrival" or . == "guest_stay" or . == "arrival_email") as $place
       | $secrets[] | "reveal:\(.):\($place)"]
    | .[]' "$manifest")"
  [[ -z "$declared" ]] && continue

  if [[ ! -f "$reasons" ]]; then
    printf '%s : permission-reasons.json manque\n' "$module" >&2
    status=1
    continue
  fi

  while IFS= read -r key; do
    for lang in fr en; do
      length="$(jq -r --arg k "$key" --arg l "$lang" \
        '.[$k][$l] // "" | gsub("^\\s+|\\s+$"; "") | length' "$reasons")"
      if (( length == 0 )); then
        printf '%s : « %s » sans justification (%s)\n' "$module" "$key" "$lang" >&2
        status=1
      elif (( length > max_chars )); then
        printf '%s : « %s » (%s) dépasse %s caractères\n' "$module" "$key" "$lang" "$max_chars" >&2
        status=1
      fi
    done
  done <<<"$declared"
done

if (( status != 0 )); then
  echo "justifiez chaque clé dans permission-reasons.json : sans elle, la stable reste en brouillon" >&2
  exit 1
fi

echo "permission-reasons.json : chaque permission déclarée est justifiée."
