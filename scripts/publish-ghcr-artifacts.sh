#!/usr/bin/env bash
# Builds OCI images (scratch + gateway.wasm + backend.jar) and pushes to GHCR.
# One package per module: ghcr.io/<owner>/portaki-module-<id>:<semver>
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARTIFACTS_ROOT="${PORTAKI_MODULES_ARTIFACTS:-$ROOT/artifacts}"

OWNER="${GITHUB_REPOSITORY_OWNER:-PortakiApp}"
OWNER_LOWER="$(echo "${OWNER}" | tr '[:upper:]' '[:lower:]')"
REGISTRY="${PORTAKI_GHCR_REGISTRY:-ghcr.io/${OWNER_LOWER}}"

if ! command -v docker >/dev/null 2>&1; then
  echo "docker required" >&2
  exit 1
fi

if [[ ! -d "${ARTIFACTS_ROOT}" ]]; then
  echo "artifacts root missing: ${ARTIFACTS_ROOT} — run install-backend-artifacts.sh and build-wasm-shim.sh first" >&2
  exit 1
fi

publish_module() {
  local module_id="$1"
  local version="$2"
  local wasm="${ARTIFACTS_ROOT}/${module_id}/${version}.wasm"
  local jar="${ARTIFACTS_ROOT}/${module_id}/${version}.jar"
  local image="${REGISTRY}/portaki-module-${module_id}:${version}"

  if [[ ! -f "${wasm}" ]]; then
    echo "skip ${module_id} (no ${wasm})"
    return 0
  fi
  if [[ ! -f "${jar}" ]]; then
    echo "skip ${module_id} (no ${jar})"
    return 0
  fi

  local build_dir
  build_dir="$(mktemp -d)"
  trap 'rm -rf "${build_dir}"' RETURN

  cp "${wasm}" "${build_dir}/gateway.wasm"
  cp "${jar}" "${build_dir}/backend.jar"

  cat > "${build_dir}/Dockerfile" <<'EOF'
FROM scratch
COPY gateway.wasm /gateway.wasm
COPY backend.jar /backend.jar
EOF

  echo "==> docker build ${image}"
  docker build -t "${image}" "${build_dir}"
  docker push "${image}"

  if [[ "${PUBLISH_GHCR_LATEST:-}" == "true" ]]; then
    docker tag "${image}" "${REGISTRY}/portaki-module-${module_id}:latest"
    docker push "${REGISTRY}/portaki-module-${module_id}:latest"
  fi

  echo "    published ${image}"
}

for module_dir in "${ARTIFACTS_ROOT}"/*; do
  [[ -d "${module_dir}" ]] || continue
  module_id="$(basename "${module_dir}")"
  if [[ "${module_id}" == _* ]]; then
    continue
  fi
  manifest="${ROOT}/modules/${module_id}/portaki.module.json"
  version="1.0.0"
  if [[ -f "${manifest}" ]]; then
    version="$(node -e "const m=require('${manifest}'); process.stdout.write(m.version||'1.0.0')")"
  fi
  publish_module "${module_id}" "${version}"
done

echo "OK — GHCR artifacts under ${REGISTRY}/portaki-module-*"
