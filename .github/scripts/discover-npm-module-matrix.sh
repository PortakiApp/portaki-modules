#!/usr/bin/env bash
# Lists @portaki/module-* packages (pnpm workspace) and writes GitHub Actions outputs:
# any, matrix (multiline JSON { include: [{ package, root, job_label }, ...] })

set -euo pipefail

DELIM="NPM_MATRIX_JSON_EOF"

to_row() {
    local root="$1"
    local pkg="$2"
    local short
    short="$(echo "${pkg}" | sed 's/^@portaki\///')"
    jq -nc --arg root "${root}" --arg package "${pkg}" --arg label "npm — ${short}" \
        '{package: $package, root: $root, job_label: $label}'
}

emit_matrix() {
    local -a roots=("$@")
    if [[ ${#roots[@]} -eq 0 ]]; then
        echo "any=false"
        echo "matrix<<${DELIM}"
        echo '{"include":[]}'
        echo "${DELIM}"
        return 0
    fi
    echo "any=true"
    local arr="["
    local first=1
    local r pkg row
    for r in "${roots[@]}"; do
        pkg="$(jq -r '.name // empty' "${r}/package.json")"
        [[ "${pkg}" == @portaki/module-* ]] || continue
        row="$(to_row "${r}" "${pkg}")"
        if [[ ${first} -eq 1 ]]; then
            first=0
        else
            arr+=","
        fi
        arr+="${row}"
    done
    if [[ "${first}" -eq 1 ]]; then
        echo "any=false"
        echo "matrix<<${DELIM}"
        echo '{"include":[]}'
        echo "${DELIM}"
        return 0
    fi
    arr+="]"
    local json
    json="$(jq -cn --argjson include "${arr}" '{include: $include}')"
    echo "matrix<<${DELIM}"
    echo "${json}"
    echo "${DELIM}"
}

declare -a pkg_roots=()
while IFS= read -r f; do
    [[ -z "${f}" ]] && continue
    root="$(dirname "${f}")"
    name="$(jq -r '.name // empty' "${f}")"
    [[ "${name}" == @portaki/module-* ]] || continue
    pkg_roots+=("${root}")
done < <(find modules -name package.json -not -path '*/node_modules/*' | sort -u)

emit_matrix "${pkg_roots[@]}"
