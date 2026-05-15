#!/usr/bin/env bash
# Écrit les sorties GitHub Actions: any, matrix (JSON pour publish Maven semver).
# Une ligne = un backend release (pas -SNAPSHOT) avec groupId / artifactId / version lus depuis le POM.

set -euo pipefail

DELIM="MAVEN_PUBLISH_MATRIX_JSON_EOF"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

to_row() {
    local java_dir="$1"
    local module_slug="$2"
    local group_id="$3"
    local artifact_id="$4"
    local pom_version="$5"
    jq -nc \
        --arg java_dir "$java_dir" \
        --arg module_slug "$module_slug" \
        --arg group_id "$group_id" \
        --arg artifact_id "$artifact_id" \
        --arg pom_version "$pom_version" \
        --arg label "maven — ${module_slug}" \
        '{java_dir: $java_dir, module_slug: $module_slug, job_label: $label, group_id: $group_id, artifact_id: $artifact_id, pom_version: $pom_version}'
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
while IFS= read -r pom; do
    [[ -z "${pom}" ]] && continue
    java_dir="$(dirname "${pom}")"
    slug="$(echo "${java_dir}" | cut -d/ -f2)"
    IFS=$'\t' read -r group_id artifact_id pom_version < <(python3 "${SCRIPT_DIR}/read-maven-pom-coords.py" "${pom}")
    if [[ "${pom_version}" == *-SNAPSHOT ]]; then
        continue
    fi
    rows+=("$(to_row "${java_dir}" "${slug}" "${group_id}" "${artifact_id}" "${pom_version}")")
done < <(find modules -path 'modules/*/backend/pom.xml' -type f 2>/dev/null | sort -u)

emit_matrix "${rows[@]}"
