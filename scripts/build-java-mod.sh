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
# AppleSkin and Fabric API ride inside the mod (fabric.mod.json "jars"). They come from Modrinth,
# pinned by SHA-256, and stay out of the repository.
fetch() {
  local path="libs/$1" url="$2" sha="$3"
  if ! echo "$sha  $path" | sha256sum --check --status 2>/dev/null; then
    mkdir -p libs
    curl -fsSL "$url" -o "$path.part"
    echo "$sha  $path.part" | sha256sum --check --quiet
    mv "$path.part" "$path"
  fi
}
fetch appleskin.jar https://cdn.modrinth.com/data/EsAfCjCV/versions/PHjDtQay/appleskin-fabric-mc26.3-3.0.10.jar \
  cb05087c39f0d860611aa1fae7cf63560214d1b71378a7c903c18dc370b5c3ce
fetch fabric-api.jar https://cdn.modrinth.com/data/P7dR8mSH/versions/nJlcvFfj/fabric-api-0.165.2%2B26.3.jar \
  2612a078a5daa06d9a14cabad40fe66de36b8cb22ee8cdf346d5d9894478c392

./gradlew --no-daemon --console=plain build

VERSION="$(sed -n 's/^mod_version=//p' gradle.properties)"
mkdir -p "$OUT"
cp "build/libs/patina-client-$VERSION.jar" "$OUT/patina-client-$MINECRAFT.jar"
echo "$OUT/patina-client-$MINECRAFT.jar"
