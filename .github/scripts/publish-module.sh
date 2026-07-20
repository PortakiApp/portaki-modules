#!/usr/bin/env bash
# Publish one module to GHCR.
# Usage: publish-module.sh <module-name>
#
# Default: build, lint, test, then publish.
# PUBLISH_FROM_ARTIFACT=1: skip build/lint/test; expect .wasm + publish-manifest.json
# already present (CI wasm job artifact) and publish with --skip-build.
set -euo pipefail

MODULE="${1:?module name required}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MODULE_DIR="${ROOT}/modules/${MODULE}"
REGISTRY="${REGISTRY:-ghcr.io/portakiapp}"
FROM_ARTIFACT="${PUBLISH_FROM_ARTIFACT:-0}"

if [[ ! -d "$MODULE_DIR" ]]; then
  echo "::error::unknown module: ${MODULE}"
  exit 1
fi

version="$(grep -E '^version\s*=' "${MODULE_DIR}/Cargo.toml" | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"

echo "=== publish ${MODULE}:${version} (from_artifact=${FROM_ARTIFACT}) ==="
cd "$MODULE_DIR"

resolve_wasm() {
  local wasm="target/wasm32-unknown-unknown/release/${MODULE}.wasm"
  if [[ ! -f "$wasm" ]]; then
    wasm="$(find target/wasm32-unknown-unknown/release -maxdepth 1 -name '*.wasm' | head -1)"
  fi
  if [[ -z "$wasm" || ! -f "$wasm" ]]; then
    echo "::error::${MODULE}: wasm artifact missing under target/wasm32-unknown-unknown/release/"
    exit 1
  fi
  printf '%s' "$wasm"
}

if [[ "$FROM_ARTIFACT" != "1" ]]; then
  rm -rf "target/wasm32-unknown-unknown/release/build/${MODULE}-"* || true
  portaki build --release
fi

WASM="$(resolve_wasm)"
if strings "$WASM" | grep -q '__wbindgen'; then
  echo "::error::${MODULE}: wasm contains wasm-bindgen imports"
  exit 1
fi

PUBLISH_MANIFEST=target/portaki/publish-manifest.json
if [[ ! -f "$PUBLISH_MANIFEST" ]]; then
  echo "::error::${MODULE}: missing ${PUBLISH_MANIFEST}"
  exit 1
fi

MANIFEST_ID="$(jq -r '.id' "$PUBLISH_MANIFEST")"
MANIFEST_VERSION="$(jq -r '.version' "$PUBLISH_MANIFEST")"
if [[ "$MANIFEST_ID" != "$MODULE" ]]; then
  echo "::error::${MODULE}: publish-manifest id=${MANIFEST_ID}"
  exit 1
fi
if [[ "$MANIFEST_VERSION" != "$version" ]]; then
  echo "::error::${MODULE}: Cargo.toml version=${version} manifest=${MANIFEST_VERSION}"
  exit 1
fi

if [[ "$FROM_ARTIFACT" != "1" ]]; then
  portaki lint
  cargo test --lib
  cargo test --test integration
fi

PUBLISH_FLAGS=(--registry "$REGISTRY")
if [[ "$FROM_ARTIFACT" == "1" ]]; then
  PUBLISH_FLAGS+=(--skip-build)
fi

PORTAKI_PUBLISH_VERSION="$version" portaki publish "${PUBLISH_FLAGS[@]}"
portaki publish "${PUBLISH_FLAGS[@]}" --dry-run

package_name="portaki-modules-${MODULE}"
package_visibility="$(gh api "orgs/PortakiApp/packages/container/${package_name}" --jq '.visibility' 2>/dev/null || echo unknown)"
echo "GHCR ${package_name} visibility: ${package_visibility}"
if [[ "${package_visibility}" != "public" ]]; then
  echo "Setting ${package_name} visibility to public..."
  if gh api --method PATCH "orgs/PortakiApp/packages/container/${package_name}" \
    --field visibility=public; then
    echo "Package ${package_name} is now public"
  else
    echo "::warning::Could not set ${package_name} visibility via API (404) — set public manually: https://github.com/orgs/PortakiApp/packages/container/package/${package_name}/settings"
  fi
fi
