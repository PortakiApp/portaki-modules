#!/usr/bin/env bash
# Usage: discover-java-backend-matrix.sh <ci|deploy>
# Writes GitHub Actions outputs: any, matrix (multiline JSON for strategy.matrix)
# ci: all modules/*/backend with pom.xml
# deploy: workflow_dispatch / FORCE_ALL -> all; else modules touched in git show HEAD

set -euo pipefail

MODE="${1:?mode ci|deploy}"
DELIM="JAVA_MATRIX_JSON_EOF"

to_row() {
    local dir="$1"
    local slug="$2"
    jq -nc --arg dir "$dir" --arg slug "$slug" --arg label "java — ${slug}" \
        '{component: "java-backend", java_dir: $dir, module_slug: $slug, job_label: $label}'
}

emit_matrix() {
    local -a dirs=("$@")
    if [[ ${#dirs[@]} -eq 0 ]]; then
        echo "any=false"
        echo "matrix<<${DELIM}"
        echo '{"include":[]}'
        echo "${DELIM}"
        return 0
    fi
    echo "any=true"
    local arr="["
    local first=1
    local d slug
    for d in "${dirs[@]}"; do
        slug="$(echo "${d}" | cut -d/ -f2)"
        local row
        row="$(to_row "${d}" "${slug}")"
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

all_dirs=()
while IFS= read -r pom; do
    [[ -z "${pom}" ]] && continue
    d="$(dirname "${pom}")"
    all_dirs+=("${d}")
done < <(find modules -path 'modules/*/backend/pom.xml' -type f 2>/dev/null | sort -u)

if [[ "${MODE}" == "ci" ]]; then
    emit_matrix "${all_dirs[@]}"
    exit 0
fi

if [[ "${GITHUB_EVENT_NAME:-}" == "workflow_dispatch" ]] || [[ "${FORCE_ALL:-}" == "1" ]]; then
    emit_matrix "${all_dirs[@]}"
    exit 0
fi

changed=()
while IFS= read -r line; do
    [[ -z "${line}" ]] && continue
    changed+=("${line}")
done < <(git show --pretty="" --name-only HEAD)
for f in "${changed[@]}"; do
    if [[ "${f}" == .github/workflows/publish-maven-central.yml ]]; then
        emit_matrix "${all_dirs[@]}"
        exit 0
    fi
done

declare -a to_deploy=()
for d in "${all_dirs[@]}"; do
    slug="$(echo "${d}" | cut -d/ -f2)"
    hit=0
    for f in "${changed[@]}"; do
        if [[ "${f}" == modules/${slug}/* ]] || [[ "${f}" == "modules/${slug}" ]]; then
            hit=1
            break
        fi
    done
    if [[ "${hit}" -eq 1 ]]; then
        to_deploy+=("${d}")
    fi
done

emit_matrix "${to_deploy[@]}"
