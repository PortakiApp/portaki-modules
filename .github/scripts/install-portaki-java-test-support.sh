#!/usr/bin/env bash
# Installs app.portaki:portaki-module-sdk-test into ~/.m2 (not on Maven Central yet).
set -euo pipefail

SDK_DIR="${PORTAKI_SDK_DIR:-.deps/portaki-sdk}"
POM="${SDK_DIR}/sdk/java-test-support/pom.xml"

if [[ ! -f "${POM}" ]]; then
  echo "::error::Missing ${POM} — checkout PortakiApp/portaki-sdk first." >&2
  exit 1
fi

echo "==> mvn install portaki-module-sdk-test from ${POM}"
mvn -B install -f "${POM}" -DskipTests
