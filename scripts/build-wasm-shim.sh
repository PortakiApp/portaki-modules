#!/usr/bin/env bash
# Builds the shared gateway Wasm shim into artifacts/{moduleId}/{version}.wasm
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SHIM_DIR="$ROOT/tools/gateway-wasm-shim"
ARTIFACTS_ROOT="${PORTAKI_MODULES_ARTIFACTS:-$ROOT/artifacts}"
TARGET="${CARGO_TARGET_DIR:-$SHIM_DIR/target}/wasm32-wasip1/release/portaki_gateway_shim.wasm"

if ! command -v cargo >/dev/null 2>&1; then
  echo "Rust toolchain required (https://rustup.rs). Install rustup, then: rustup target add wasm32-wasip1" >&2
  exit 1
fi

rustup target add wasm32-wasip1 >/dev/null 2>&1 || true

echo "==> cargo build (wasm32-wasip1)…"
cargo build --manifest-path "$SHIM_DIR/Cargo.toml" --release --target wasm32-wasip1

mkdir -p "$ARTIFACTS_ROOT/_shim"
cp "$TARGET" "$ARTIFACTS_ROOT/_shim/portaki-gateway-shim.wasm"

WASM_MODULES="${PORTAKI_WASM_BACKEND_MODULES:-sections,rules,appliances,pre-arrival-form,checklist,train,events,ical-sync,trmnl}"
IFS=',' read -r -a MODULE_IDS <<< "$WASM_MODULES"
for module_id in "${MODULE_IDS[@]}"; do
  module_id="$(echo "$module_id" | xargs)"
  if [[ -z "$module_id" ]]; then
    continue
  fi
  manifest="$ROOT/modules/$module_id/portaki.module.json"
  version="1.0.0"
  if [[ -f "$manifest" ]]; then
    version="$(node -e "const m=require('$manifest'); process.stdout.write(m.version||'1.0.0')")"
  fi
  dest_dir="$ARTIFACTS_ROOT/$module_id"
  mkdir -p "$dest_dir"
  cp "$TARGET" "$dest_dir/$version.wasm"
  echo "==> installed $dest_dir/$version.wasm"
done

echo "OK — gateway shim at $ARTIFACTS_ROOT/_shim/portaki-gateway-shim.wasm"
