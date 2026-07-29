#!/usr/bin/env bash
# Discover modules/*/Cargo.toml + portaki.module.json and regenerate
# release-please-config.json + .release-please-manifest.json.
#
# Source of truth for package paths: the filesystem (this script).
# Do not hand-edit package names in release-please-config.json.
#
# Usage:
#   scripts/generate-release-please-config.sh           # write files
#   scripts/generate-release-please-config.sh --check   # exit 1 on drift
#
# Env:
#   BOOTSTRAP_SHA  -- set once on first bootstrap (preserved afterwards)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

CHECK=0
if [[ "${1:-}" == "--check" ]]; then
  CHECK=1
fi

CONFIG_PATH="release-please-config.json"
MANIFEST_PATH=".release-please-manifest.json"

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "error: required command not found: $1" >&2
    exit 1
  }
}

need_cmd jq
need_cmd python3

cargo_version() {
  local cargo_toml="$1"
  python3 - "$cargo_toml" <<'PY'
import re, sys
text = open(sys.argv[1], encoding="utf-8").read()
m = re.search(r'(?m)^version\s*=\s*"([^"]+)"', text)
if not m:
    sys.exit(f"missing version in {sys.argv[1]}")
print(m.group(1))
PY
}

json_version() {
  local json_path="$1"
  jq -r '.version // empty' "$json_path"
}

# Discover releasable modules: both Cargo.toml and portaki.module.json required.
MODULES_FILE="$(mktemp)"
trap 'rm -f "$MODULES_FILE" "$TMP_CONFIG" "$TMP_MANIFEST" "$TMP_DIR"/* 2>/dev/null; rmdir "$TMP_DIR" 2>/dev/null || true' EXIT
TMP_DIR="$(mktemp -d)"
TMP_CONFIG="${TMP_DIR}/release-please-config.json"
TMP_MANIFEST="${TMP_DIR}/.release-please-manifest.json"

: >"$MODULES_FILE"
for cargo in modules/*/Cargo.toml; do
  [[ -f "$cargo" ]] || continue
  mod_dir="$(dirname "$cargo")"
  mod_id="$(basename "$mod_dir")"
  manifest_json="${mod_dir}/portaki.module.json"
  if [[ ! -f "$manifest_json" ]]; then
    echo "skip ${mod_id}: missing portaki.module.json" >&2
    continue
  fi
  ver="$(cargo_version "$cargo")"
  jver="$(json_version "$manifest_json")"
  if [[ -z "$jver" ]]; then
    echo "error: ${manifest_json}: missing .version" >&2
    exit 1
  fi
  if [[ "$ver" != "$jver" ]]; then
    echo "error: ${mod_id}: Cargo.toml version=${ver} portaki.module.json version=${jver}" >&2
    exit 1
  fi
  printf '%s\t%s\n' "$mod_id" "$ver" >>"$MODULES_FILE"
done

if [[ ! -s "$MODULES_FILE" ]]; then
  echo "error: no releasable modules found under modules/*/" >&2
  exit 1
fi

EXISTING_BOOTSTRAP=""
EXISTING_MANIFEST="{}"
if [[ -f "$CONFIG_PATH" ]]; then
  EXISTING_BOOTSTRAP="$(jq -r '."bootstrap-sha" // empty' "$CONFIG_PATH")"
fi
if [[ -f "$MANIFEST_PATH" ]]; then
  EXISTING_MANIFEST="$(cat "$MANIFEST_PATH")"
fi

BOOTSTRAP="${BOOTSTRAP_SHA:-$EXISTING_BOOTSTRAP}"

# Build packages object + merged manifest via Python for stable key order.
python3 - "$MODULES_FILE" "$EXISTING_MANIFEST" "$BOOTSTRAP" "$TMP_CONFIG" "$TMP_MANIFEST" <<'PY'
import json
import sys
from collections import OrderedDict

modules_path, existing_manifest_raw, bootstrap, out_config, out_manifest = sys.argv[1:6]

modules = []
with open(modules_path, encoding="utf-8") as f:
    for line in f:
        line = line.rstrip("\n")
        if not line:
            continue
        mod_id, ver = line.split("\t", 1)
        modules.append((mod_id, ver))

modules.sort(key=lambda x: x[0])

existing_manifest = json.loads(existing_manifest_raw) if existing_manifest_raw.strip() else {}

changelog_sections = [
    {"type": "feat", "section": "Features"},
    {"type": "fix", "section": "Bug Fixes"},
    {"type": "perf", "section": "Performance"},
    {"type": "revert", "section": "Reverts"},
    {"type": "docs", "section": "Documentation", "hidden": True},
    {"type": "style", "section": "Styling", "hidden": True},
    {"type": "chore", "section": "Miscellaneous", "hidden": True},
    {"type": "refactor", "section": "Miscellaneous", "hidden": True},
    {"type": "test", "section": "Miscellaneous", "hidden": True},
    {"type": "build", "section": "Miscellaneous", "hidden": True},
    {"type": "ci", "section": "Miscellaneous", "hidden": True},
]

packages = OrderedDict()
manifest = OrderedDict()

for mod_id, ver in modules:
    pkg_path = f"modules/{mod_id}"
    packages[pkg_path] = {
        "package-name": mod_id,
        "component": mod_id,
        "release-type": "simple",
        "changelog-path": "CHANGELOG.md",
        "include-component-in-tag": True,
        "include-v-in-tag": True,
        "include-v-in-release-name": True,
        # Paths are relative to the package directory; release-please
        # prefixes them with the package path itself.
        "extra-files": [
            {
                "type": "toml",
                "path": "Cargo.toml",
                "jsonpath": "$.package.version",
            },
            {
                "type": "json",
                "path": "portaki.module.json",
                "jsonpath": "$.version",
            },
        ],
        "changelog-sections": changelog_sections,
    }
    # Preserve prior released versions; seed new packages from Cargo.toml.
    if pkg_path in existing_manifest and existing_manifest[pkg_path]:
        manifest[pkg_path] = existing_manifest[pkg_path]
    else:
        manifest[pkg_path] = ver

config = OrderedDict()
config["$schema"] = (
    "https://raw.githubusercontent.com/googleapis/release-please/main/schemas/config.json"
)
if bootstrap:
    config["bootstrap-sha"] = bootstrap
config["label"] = "pending release"
config["release-label"] = "released"
config["always-update"] = True
config["draft-pull-request"] = True
# Independent package versions; one PR listing only packages that need a bump.
config["separate-pull-requests"] = False
# release-please requires ${scope} in the title pattern to round-trip its own PR;
# for a grouped release it renders empty, so the visible title stays "chore: release modules".
config["group-pull-request-title-pattern"] = "chore${scope}: release modules"
config["pull-request-header"] = "Release candidate for Portaki modules"
config["pull-request-footer"] = (
    "## Before merge\n\n"
    "- [ ] CHANGELOG entries look correct per module\n"
    "- [ ] `Cargo.toml` and `portaki.module.json` versions stay in sync "
    "(release-please bumps both)\n"
    "- [ ] After merge, `ci` publish on `main` should push GHCR tags from "
    "bumped Cargo.toml versions\n\n"
    "Config is generated by `scripts/generate-release-please-config.sh` -- "
    "do not hand-edit package paths."
)
config["packages"] = packages

with open(out_config, "w", encoding="utf-8") as f:
    json.dump(config, f, indent=2)
    f.write("\n")

with open(out_manifest, "w", encoding="utf-8") as f:
    json.dump(manifest, f, indent=2)
    f.write("\n")

print(f"discovered {len(modules)} module(s): " + ", ".join(m for m, _ in modules))
PY

if [[ "$CHECK" -eq 1 ]]; then
  drift=0
  if ! cmp -s "$TMP_CONFIG" "$CONFIG_PATH" 2>/dev/null; then
    echo "drift: $CONFIG_PATH is out of date (run scripts/generate-release-please-config.sh)" >&2
    drift=1
  fi
  if ! cmp -s "$TMP_MANIFEST" "$MANIFEST_PATH" 2>/dev/null; then
    echo "drift: $MANIFEST_PATH is out of date (run scripts/generate-release-please-config.sh)" >&2
    drift=1
  fi
  exit "$drift"
fi

cp "$TMP_CONFIG" "$CONFIG_PATH"
cp "$TMP_MANIFEST" "$MANIFEST_PATH"
echo "wrote $CONFIG_PATH"
echo "wrote $MANIFEST_PATH"
