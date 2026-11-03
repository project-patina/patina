#!/bin/sh
# Builds Patina for Windows from Linux: a folder with the launcher and the engine beside it.
# Needs zig on PATH, cargo-zigbuild, the Rust target x86_64-pc-windows-gnu, and the
# game prepared once on this computer (the exporters are compiled from it).
set -eu

target=x86_64-pc-windows-gnu
# zig turns the date macros in C dependencies into errors.
export CFLAGS_x86_64_pc_windows_gnu=-Wno-date-time
repo=$(cd "$(dirname "$0")/.." && pwd)
data=${XDG_DATA_HOME:-$HOME/.local/share}/amethyst
out="$repo/dist-windows"
package="$out/Patina"
stage="$package/engine"

# The launcher embeds it when it exists; a release must not ship without it.
if [ ! -f "$repo/resources/patina-mod/patina-client-26.3.jar" ]; then
    echo "The integrated mod is missing: run scripts/build-java-mod.sh" >&2
    exit 1
fi

rm -rf "$out"
mkdir -p "$stage/mods/patina/src/test" "$stage/exporters" "$out/.tools"

# The icon and version info go in through windres, which zig stands in for.
cat > "$out/.tools/x86_64-w64-mingw32-windres" <<'WINDRES'
#!/bin/sh
includes=""
while [ $# -gt 0 ]; do
    case "$1" in
        -V | --version) echo "GNU windres (zig rc)"; exit 0 ;;
        --input) input=$2; shift 2 ;;
        --output) output=$2; shift 2 ;;
        --include-dir | -I) includes="$includes /i $2"; shift 2 ;;
        *) shift ;;
    esac
done
exec zig rc /:output-format coff /:target x64 /fo "$output" $includes -- "$input"
WINDRES
chmod +x "$out/.tools/x86_64-w64-mingw32-windres"
export PATH="$out/.tools:$PATH"

cd "$repo/engine"
cargo zigbuild -j 6 --release --locked --target "$target" -p patina-client --bin patina-client -p patina-prepare --bin patina-prepare
cp "target/$target/release/patina-client.exe" "target/$target/release/patina-prepare.exe" research-lock.json LICENSE THIRD_PARTY_NOTICES.md "$stage/"
cp -r mods/patina/src/test/java "$stage/mods/patina/src/test/java"

# Players have a Java runtime but may have no compiler, so the exporters ship compiled.
compiled=$(JAVA_HOME="$data/runtimes/java-runtime-epsilon" "$data/engine/patina-prepare" compile-exporters --root "$data/engine" | tail -1)
cp -r "$compiled/server" "$compiled/client" "$compiled/manifest.json" "$stage/exporters/"

cd "$repo"
npx tauri build --no-bundle --runner cargo-zigbuild --target "$target" -- -j 6
cp "target/$target/release/amethyst-launcher.exe" "$package/Patina.exe"
# The GNU build links WebView2's loader dynamically: Windows wants it beside the program.
cp "target/$target/release/WebView2Loader.dll" "$package/"
rm -rf "$out/.tools"
python3 "$repo/scripts/collect-licenses.py" "$package"
python3 - "$out" <<'PY'
import os, sys, zipfile
out = sys.argv[1]
with zipfile.ZipFile(os.path.join(out, "Patina-windows.zip"), "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
    for folder, _, files in os.walk(os.path.join(out, "Patina")):
        for name in files:
            path = os.path.join(folder, name)
            z.write(path, os.path.relpath(path, out))
PY
(cd "$package" && sha256sum Patina.exe WebView2Loader.dll engine/patina-client.exe engine/patina-prepare.exe) > "$out/SHA256SUMS.txt"
ls -la "$out"
