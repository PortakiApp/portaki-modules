#!/usr/bin/env bash
# Builds module *-backend JARs and installs them under artifacts/{moduleId}/{version}.jar
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARTIFACTS_ROOT="${PORTAKI_MODULES_ARTIFACTS:-$ROOT/artifacts}"

BACKEND_MODULES=(
  sections
  rules
  appliances
  pre-arrival-form
  checklist
  train
  events
  ical-sync
  trmnl
)

for module_id in "${BACKEND_MODULES[@]}"; do
  backend_dir="$ROOT/modules/$module_id/backend"
  manifest="$ROOT/modules/$module_id/portaki.module.json"
  if [[ ! -f "$backend_dir/pom.xml" ]]; then
    echo "skip $module_id (no backend/pom.xml)"
    continue
  fi
  version="1.0.0"
  if [[ -f "$manifest" ]]; then
    version="$(node -e "const m=require('$manifest'); process.stdout.write(m.version||'1.0.0')")"
  fi
  echo "==> mvn package $module_id-backend @ $version"
  (cd "$backend_dir" && mvn -q package -DskipTests)
  jar="$(find "$backend_dir/target" -maxdepth 1 -name '*.jar' ! -name '*-sources.jar' ! -name '*-javadoc.jar' | head -1)"
  if [[ -z "$jar" ]]; then
    echo "no jar produced for $module_id" >&2
    exit 1
  fi
  dest_dir="$ARTIFACTS_ROOT/$module_id"
  mkdir -p "$dest_dir"
  cp "$jar" "$dest_dir/$version.jar"
  echo "    -> $dest_dir/$version.jar"
done

echo "OK — backend JARs under $ARTIFACTS_ROOT"
