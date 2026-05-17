#!/usr/bin/env bash
# Modules with backend JAR + wasm shim artifacts → GHCR OCI publish matrix.
set -euo pipefail

DELIM="OCI_MATRIX_JSON_EOF"

to_row() {
  local slug="$1"
  jq -nc --arg slug "$slug" --arg label "oci — ${slug}" \
    '{module_slug: $slug, job_label: $label}'
}

emit_matrix() {
  local -a slugs=("$@")
  if [[ ${#slugs[@]} -eq 0 ]]; then
    echo "any=false"
    echo "matrix<<${DELIM}"
    echo '{"include":[]}'
    echo "${DELIM}"
    return 0
  fi
  echo "any=true"
  local arr="["
  local first=1
  local slug row
  for slug in "${slugs[@]}"; do
    row="$(to_row "${slug}")"
    if [[ ${first} -eq 1 ]]; then
      first=0
    else
      arr+=","
    fi
    arr+="${row}"
  done
  arr+="]"
  local json
  json="$(jq -cn --argjson include "${arr}" '{include: $include}')"
  echo "matrix<<${DELIM}"
  echo "${json}"
  echo "${DELIM}"
}

all_slugs=()
while IFS= read -r pom; do
  [[ -z "${pom}" ]] && continue
  slug="$(echo "$(dirname "${pom}")" | cut -d/ -f2)"
  all_slugs+=("${slug}")
done < <(find modules -path 'modules/*/backend/pom.xml' -type f 2>/dev/null | sort -u)

if [[ "${GITHUB_EVENT_NAME:-}" == "workflow_dispatch" ]] || [[ "${FORCE_ALL_OCI:-}" == "1" ]]; then
  emit_matrix "${all_slugs[@]}"
  exit 0
fi

changed=()
while IFS= read -r line; do
  [[ -z "${line}" ]] && continue
  changed+=("${line}")
done < <(git show --pretty="" --name-only HEAD 2>/dev/null || true)

if [[ ${#changed[@]} -eq 0 ]]; then
  emit_matrix "${all_slugs[@]}"
  exit 0
fi

for f in "${changed[@]}"; do
  if [[ "${f}" == .github/workflows/publish-ghcr-artifacts.yml ]] \
    || [[ "${f}" == scripts/publish-ghcr-artifacts.sh ]] \
    || [[ "${f}" == scripts/install-backend-artifacts.sh ]]; then
    emit_matrix "${all_slugs[@]}"
    exit 0
  fi
done

declare -a to_publish=()
for slug in "${all_slugs[@]}"; do
  hit=0
  for f in "${changed[@]}"; do
    if [[ "${f}" == modules/${slug}/* ]] || [[ "${f}" == "modules/${slug}" ]]; then
      hit=1
      break
    fi
  done
  if [[ "${hit}" -eq 1 ]]; then
    to_publish+=("${slug}")
  fi
done

emit_matrix "${to_publish[@]}"
