#!/bin/sh
# Builds the update packages for VERSION: dist-updates/patina-VERSION-<platform>.zip,
# each with launcher/ and engine/ at the top. scripts/publish-update.py uploads them.
#
# Needs everything scripts/build-windows.sh needs, including the game prepared once on
# this computer. The compiled exporters of the Windows build ship in both packages: they
# are plain Patina classes, and players have a Java runtime but may have no compiler.
#
# --zip-only packs the trees an earlier run left in dist-updates/stage again.
set -eu

usage="usage: package-update.sh VERSION [--zip-only]"
version=${1:?$usage}
mode=${2:-build}
repo=$(cd "$(dirname "$0")/.." && pwd)
out="$repo/dist-updates"
stage="$out/stage"

packaged=$(python3 -c 'import json, sys; print(json.load(open(sys.argv[1]))["version"])' "$repo/package.json")
if [ "$version" != "$packaged" ]; then
    echo "package.json says $packaged, not $version" >&2
    exit 1
fi
# Compiled in, these would point the released launcher at a test server.
if [ -n "${PATINA_TEST_UPDATE_BASE:-}${PATINA_TEST_VERSION:-}${PATINA_DEV:-}${VITE_PATINA_DEV:-}" ]; then
    echo "Unset PATINA_TEST_UPDATE_BASE, PATINA_TEST_VERSION, PATINA_DEV and VITE_PATINA_DEV first" >&2
    exit 1
fi
# The launcher embeds it when it exists; a release must not ship without it.
if [ ! -f "$repo/resources/patina-mod/patina-client-26.3.jar" ]; then
    echo "The integrated mod is missing: run scripts/build-java-mod.sh" >&2
    exit 1
fi

if [ "$mode" != "--zip-only" ]; then
    if [ "$mode" != build ]; then
        echo "$usage" >&2
        exit 1
    fi
    rm -rf "$stage"
    mkdir -p "$stage/linux/launcher" "$stage/windows/launcher"
    # Update packages stay out of Git, whatever else this folder gets used for.
    printf '*\n' > "$out/.gitignore"

    sh "$repo/scripts/build-windows.sh"
    cp -r "$repo/dist-windows/Patina/engine" "$stage/windows/engine"
    cp "$repo/dist-windows/Patina/Patina.exe" "$stage/windows/launcher/patina.exe"
    cp "$repo/dist-windows/Patina/WebView2Loader.dll" "$stage/windows/launcher/"
    cp -r "$repo/dist-windows/Patina/LICENSE" "$repo/dist-windows/Patina/LICENSES" "$repo/dist-windows/Patina/THIRD_PARTY_NOTICES.md" "$stage/windows/launcher/"

    cd "$repo"
    npm run build
    cargo build -j 6 --release --locked -p amethyst-launcher --features custom-protocol
    cp target/release/amethyst-launcher "$stage/linux/launcher/patina"
    AMETHYST_ENGINE_DIR="$stage/linux/engine" sh "$repo/scripts/install-engine.sh"
    cp -r "$repo/dist-windows/Patina/engine/exporters" "$stage/linux/engine/exporters"
    python3 "$repo/scripts/collect-licenses.py" "$stage/linux"
    mv "$stage/linux/LICENSE" "$stage/linux/LICENSES" "$stage/linux/THIRD_PARTY_NOTICES.md" "$stage/linux/launcher/"
fi

python3 - "$stage" "$out" "$version" <<'PY'
import os, re, sys, zipfile

stage, out, version = sys.argv[1:]
# Game files come from the player's own copy. Only Patina's classes may be compiled in.
BLOCKED = re.compile(r"(^|/)\.local/|(^|/)assets/minecraft/|\.jar$")
CLASSES = re.compile(r"engine/exporters/(server|client)/dev/patinalab/parity/[^/]+\.class")

for platform, suffix in [("linux", ""), ("windows", ".exe")]:
    root = os.path.join(stage, platform)
    names = []
    for folder, _, files in os.walk(root):
        for name in files:
            names.append(os.path.relpath(os.path.join(folder, name), root).replace(os.sep, "/"))
    names.sort()
    for name in names:
        if BLOCKED.search(name) or (name.endswith(".class") and not CLASSES.fullmatch(name)):
            sys.exit(f"Refusing to package {platform}/{name}")
        if os.path.islink(os.path.join(root, name)):
            sys.exit(f"Refusing to package a link: {platform}/{name}")
        if name.split("/")[0] not in ("launcher", "engine"):
            sys.exit(f"Outside launcher/ and engine/: {platform}/{name}")
    required = [f"launcher/patina{suffix}", f"engine/patina-client{suffix}", f"engine/patina-prepare{suffix}", "launcher/LICENSE",
                "engine/LICENSE", "engine/research-lock.json", "engine/exporters/manifest.json"]
    missing = [name for name in required if name not in names]
    if missing:
        sys.exit(f"Missing from the {platform} package: {', '.join(missing)}")
    target = os.path.join(out, f"patina-{version}-{platform}-x86_64.zip")
    with zipfile.ZipFile(target, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
        for name in names:
            z.write(os.path.join(root, name), name)
    print(f"{target}: {os.path.getsize(target)} bytes, {len(names)} files")
PY
