#!/usr/bin/env bash
# Écrit les sorties GitHub Actions: any, matrix (JSON strategy.matrix pour publish npm semver).
# Une ligne matrix = un package @portaki/module-* (modules/*/package.json + pre-arrival-form/frontend).

set -euo pipefail

DELIM="NPM_PUBLISH_MATRIX_JSON_EOF"

to_row() {
    local pkg_dir="$1"
    local module_slug="$2"
    local npm_name="$3"
    local version="$4"
    jq -nc \
        --arg pkg_dir "$pkg_dir" \
        --arg module_slug "$module_slug" \
        --arg npm_name "$npm_name" \
        --arg version "$version" \
        --arg label "npm — ${module_slug}" \
        '{npm_package_name: $npm_name, pkg_dir: $pkg_dir, module_slug: $module_slug, job_label: $label, local_version: $version}'
}

emit_empty() {
    echo "any=false"
    echo "matrix<<${DELIM}"
    echo '{"include":[]}'
    echo "${DELIM}"
}

emit_matrix() {
    local -a rows=("$@")
    if [[ ${#rows[@]} -eq 0 ]]; then
        emit_empty
        return 0
    fi
    echo "any=true"
    local arr="["
    local first=1
    local r
    for r in "${rows[@]}"; do
        if [[ ${first} -eq 1 ]]; then
            first=0
        else
            arr+=","
        fi
        arr+="${r}"
    done
    arr+="]"
    local json
    json="$(jq -cn --argjson include "${arr}" '{include: $include}')"
    echo "matrix<<${DELIM}"
    echo "${json}"
    echo "${DELIM}"
}

rows=()
while IFS= read -r -d '' pj; do
    [[ -z "${pj}" ]] && continue
    name="$(jq -r '.name // empty' "${pj}")"
    [[ "${name}" == @portaki/module-* ]] || continue
    ver="$(jq -r '.version // empty' "${pj}")"
    [[ -n "${ver}" ]] || continue
    dir="$(dirname "${pj}")"
    rel="${dir#modules/}"
    slug="${rel//\//-}"
    rows+=("$(to_row "${dir}" "${slug}" "${name}" "${ver}")")
done < <(find modules -name package.json -type f ! -path '*/node_modules/*' -print0 2>/dev/null)

emit_matrix "${rows[@]}"
