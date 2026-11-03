#!/usr/bin/env bash
# Builds the Patina Fabric mod and places it where the launcher bundles it from.
# Needs JDK 25: set JAVA_HOME, or keep a Temurin 25 under ~/.local/opt.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/resources/patina-mod"
MINECRAFT=26.3

if [ -z "${JAVA_HOME:-}" ]; then
  for candidate in "$HOME"/.local/opt/jdk-25*; do
    [ -x "$candidate/bin/javac" ] && JAVA_HOME="$candidate"
  done
fi
if [ -z "${JAVA_HOME:-}" ]; then
  echo "JDK 25 not found: set JAVA_HOME" >&2
  exit 1
fi
export JAVA_HOME

cd "$ROOT/java-mod"
./gradlew --no-daemon --console=plain build

VERSION="$(sed -n 's/^mod_version=//p' gradle.properties)"
mkdir -p "$OUT"
cp "build/libs/patina-client-$VERSION.jar" "$OUT/patina-client-$MINECRAFT.jar"
echo "$OUT/patina-client-$MINECRAFT.jar"
